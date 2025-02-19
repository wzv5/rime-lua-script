--[[
功能
* 在后台自动记录剪切板历史，保存最近 5 次的内容
* 输入设置的关键字之后显示剪切板历史，一键输入

https://github.com/wzv5/rime-lua-script

依赖
https://github.com/wzv5/rime-lua-script/blob/main/bin/windows-x64/lua_helper_wzv5.dll
下载后放入 weasel 安装目录中，如 C:\Program Files\Rime\weasel-0.16.3。

用法
patch:
  engine/translators/@before 0: lua_translator@*clipboard_translator
需要插入到尽可能靠前的位置，避免被其他组件降低权重。
]]

local helper = require("lua_helper_wzv5")
local translator = {}

function translator.init(env)
    local config = env.engine.schema.config
    env.name_space = env.name_space:gsub('^*', '')
    env.keyword = config:get_string(env.name_space .. "/keyword") or "jqb"
    helper.clipboard.init()
end

function translator.fini(env)
    helper.clipboard.fini()
end

function translator.func(input, seg, env)
    if input == env.keyword then
        local data = helper.clipboard.get()
        for _, value in ipairs(data) do
            local c = Candidate("", seg.start, seg._end, value, "")
            c.quality = 100
            yield(c)
        end
    end
end

return translator
