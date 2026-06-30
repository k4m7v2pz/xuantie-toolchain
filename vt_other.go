//go:build !windows

package main

// 非 Windows 平台无需启用虚拟终端处理 (原生支持 ANSI 转义)
func enableVirtualTerminalProcessingImpl() {}
