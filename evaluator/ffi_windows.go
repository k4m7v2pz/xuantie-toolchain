//go:build windows

package evaluator

import (
	"strings"
	"syscall"
	"unsafe"

	"xuantie/object"
)

// ffiLoadDLL 加载 Windows DLL,返回句柄
func ffiLoadDLL(libPath string) (uintptr, error) {
	dll, err := syscall.LoadDLL(libPath)
	if err != nil {
		return 0, err
	}
	return uintptr(dll.Handle), nil
}

// ffiNewDLLBuiltin 创建一个调用 DLL 过程的 Builtin 函数
func ffiNewDLLBuiltin(path string, ptr uintptr, procName string) *object.Builtin {
	return &object.Builtin{
		Fn: func(fArgs ...object.Object) object.Object {
			dll := &syscall.DLL{Name: path, Handle: syscall.Handle(ptr)}
			proc, err := dll.FindProc(procName)
			if err != nil {
				return &object.Result{IsSuccess: false, Error: &object.String{Value: err.Error()}}
			}

			uArgs := make([]uintptr, len(fArgs))
			for i, a := range fArgs {
				switch v := a.(type) {
				case *object.Integer:
					uArgs[i] = uintptr(v.Value)
				case *object.String:
					if strings.HasSuffix(procName, "W") {
						p, _ := syscall.UTF16PtrFromString(v.Value)
						uArgs[i] = uintptr(unsafe.Pointer(p))
					} else {
						p, _ := syscall.BytePtrFromString(v.Value)
						uArgs[i] = uintptr(unsafe.Pointer(p))
					}
				default:
					uArgs[i] = 0
				}
			}

			r1, _, _ := proc.Call(uArgs...)
			return &object.Integer{Value: int64(r1)}
		},
	}
}

// ffiApplyFFI 调用 FFIFunction
func ffiApplyFFI(function *object.FFIFunction, args []object.Object) object.Object {
	dll := &syscall.DLL{Name: function.Path, Handle: syscall.Handle(function.Handle)}
	proc, err := dll.FindProc(function.Name)
	if err != nil {
		return &object.Result{IsSuccess: false, Error: &object.String{Value: err.Error()}}
	}

	uArgs := make([]uintptr, len(args))
	for i, a := range args {
		switch v := a.(type) {
		case *object.Integer:
			uArgs[i] = uintptr(v.Value)
		case *object.String:
			if strings.HasSuffix(function.Name, "W") {
				p, _ := syscall.UTF16PtrFromString(v.Value)
				uArgs[i] = uintptr(unsafe.Pointer(p))
			} else {
				p, _ := syscall.BytePtrFromString(v.Value)
				uArgs[i] = uintptr(unsafe.Pointer(p))
			}
		default:
			uArgs[i] = 0
		}
	}

	r1, _, _ := proc.Call(uArgs...)
	return &object.Integer{Value: int64(r1)}
}
