## 编译方法

设置环境变量 `LUA_LIB` 和 `LUA_LIB_NAME`，如：

```powershell
$env:LUA_LIB = "C:\librime"
$env:LUA_LIB_NAME = "rime"
```

然后运行 `cargo build --release`。

`rime.lib` 可从 [librime 官方仓库](https://github.com/rime/librime/releases) 下载 `rime-xxxx-Windows-msvc-x64.7z`，位于压缩包内 `dist/lib` 目录下。
