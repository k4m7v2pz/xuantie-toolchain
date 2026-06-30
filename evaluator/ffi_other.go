//go:build !windows

package evaluator

import (
	"fmt"

	"xuantie/object"
)

// ffiLoadDLL 非 Windows 平台不支持 DLL 加载
func ffiLoadDLL(libPath string) (uintptr, error) {
	return 0, fmt.Errorf("DLL 加载仅支持 Windows 平台")
}

// ffiNewDLLBuiltin 非 Windows 平台返回始终报错的 Builtin
func ffiNewDLLBuiltin(path string, ptr uintptr, procName string) *object.Builtin {
	return &object.Builtin{
		Fn: func(fArgs ...object.Object) object.Object {
			return &object.Result{IsSuccess: false, Error: &object.String{Value: "DLL 调用仅支持 Windows 平台"}}
		},
	}
}

// ffiApplyFFI 非 Windows 平台返回错误
func ffiApplyFFI(function *object.FFIFunction, args []object.Object) object.Object {
	return &object.Result{IsSuccess: false, Error: &object.String{Value: "FFI 调用仅支持 Windows 平台"}}
}
