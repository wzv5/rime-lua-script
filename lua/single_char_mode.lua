--[[
功能：
* 按 0 切换单字模式
* 选字或退出输入状态后自动关闭单字模式

https://github.com/wzv5/rime-lua-script

用法：
patch:
  engine/filters/+:
    - lua_filter@*single_char_mode*filter
  engine/processors/@before 6: lua_processor@*single_char_mode*processor

注意，processor 需要插入到 selector 之前：
engine:
  processors:
    ...
    - punctuator
    - lua_processor@*single_char_mode*processor
    - selector
    ...
@before 6 可用于白霜、雾凇等大部分方案，
最好直接改 xxx.schema.yaml。
]]

local SINGLE_CHAR_MODE = "single_char_mode"

local function off(ctx)
  -- 减少调用次数，不然一直写日志
  if ctx:get_option(SINGLE_CHAR_MODE) then
    ctx:set_option(SINGLE_CHAR_MODE, false)
  end
end

---@param key KeyEvent
---@param env Env
local function processor(key, env)
  local engine = env.engine
  local context = engine.context
  -- 参考 librime/src/rime/gear/selector.cc > ProcessKeyEvent
  if context.composition:empty() then
    return 2
  end
  local seg = context.composition:back()
  if not seg.menu or seg:has_tag("raw") then
    return 2
  end
  -- 候选词个数 > 9，与单字模式冲突
  if engine.schema.page_size > 9 then
    return 2
  end
  -- 按 0 切换单字模式
  if key:repr() == "0" and context:is_composing() then
    context:set_option(SINGLE_CHAR_MODE, not context:get_option(SINGLE_CHAR_MODE))
    return 1
  end
  return 2
end

local filter = {}

function filter.init(env)
  local context = env.engine.context
  context:set_option(SINGLE_CHAR_MODE, false)
  -- 选字之后关闭单字模式
  env.notifier1 = context.select_notifier:connect(function(ctx)
    off(ctx)
  end)
  -- 按 esc 取消输入或切换 ascii 模式时关闭单字模式
  env.notifier2 = context.update_notifier:connect(function(ctx)
    if not ctx:is_composing() then
      off(ctx)
    end
  end)
end

function filter.fini(env)
  env.notifier1:disconnect()
  env.notifier2:disconnect()
end

---@param input Translation
---@param env Env
function filter.func(input, env)
  local context = env.engine.context
  local on = context:get_option(SINGLE_CHAR_MODE)
  for cand in input:iter() do
    if not on or utf8.len(cand.text) == 1 then
      yield(cand)
    end
  end
end

return { processor = processor, filter = filter }
