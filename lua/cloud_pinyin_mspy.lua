--[[
功能：
* 微软双拼，按 Ctrl+Shift+C 调用指定搜索引擎的建议词列表
* 选词上屏后，如果输入拼音与反查结果一致，就加入词库，下次就可直接输入
* 通过 rust 实现同时并发请求多个搜索引擎，并整合所有结果
* 支持的搜索引擎：baidu、bilibili、bing、taobao

https://github.com/wzv5/rime-lua-script

依赖：
https://github.com/wzv5/rime-lua-script/blob/main/bin/windows-x64/lua_helper_wzv5.dll
下载后放入 weasel 安装目录中，如 C:\Program Files\Rime\weasel-0.17.4。

用法：
patch:
  engine/processors/+:
    - lua_processor@*cloud_pinyin_mspy*processor
  engine/translators/+:
    - lua_translator@*cloud_pinyin_mspy*translator
  cloud_pinyin:
    # 可选，设置反查词库，如果不指定，则不反查，也不会自动加入词库
    reverse_db: rime_frost
    # 可选，设置想要使用的搜索引擎，默认为 baidu
    providers:
      - baidu
      - bilibili
    # 可选，对结果进行额外处理，可用值：none（仅去重）、sort_by_length（按长度排序）、truncate（默认，截断为输入的长度）
    post_processing: truncate

参考：https://github.com/hchunhui/librime-cloud/issues/14#issuecomment-2222450807
]]

local helper = require("lua_helper_wzv5")

local flag = false

local mspy2qp_table = { ["oa"] = "a", ["ol"] = "ai", ["oj"] = "an", ["oh"] = "ang", ["ok"] = "ao", ["ba"] = "ba", ["bl"] = "bai", ["bj"] = "ban", ["bh"] = "bang", ["bk"] = "bao", ["bz"] = "bei", ["bf"] = "ben", ["bg"] = "beng", ["bi"] = "bi", ["bm"] = "bian", ["bd"] = "biang", ["bc"] = "biao", ["bx"] = "bie", ["bn"] = "bin", ["b;"] = "bing", ["bo"] = "bo", ["bu"] = "bu", ["ca"] = "ca", ["cl"] = "cai", ["cj"] = "can", ["ch"] = "cang", ["ck"] = "cao", ["ce"] = "ce", ["cz"] = "cei", ["cf"] = "cen", ["cg"] = "ceng", ["ia"] = "cha", ["il"] = "chai", ["ij"] = "chan", ["ih"] = "chang", ["ik"] = "chao", ["ie"] = "che", ["if"] = "chen", ["ig"] = "cheng", ["ii"] = "chi", ["is"] = "chong", ["ib"] = "chou", ["iu"] = "chu", ["iw"] = "chua", ["iy"] = "chuai", ["ir"] = "chuan", ["id"] = "chuang", ["iv"] = "chui", ["ip"] = "chun", ["io"] = "chuo", ["ci"] = "ci", ["cs"] = "cong", ["cb"] = "cou", ["cu"] = "cu", ["cr"] = "cuan", ["cv"] = "cui", ["cp"] = "cun", ["co"] = "cuo", ["da"] = "da", ["dl"] = "dai", ["dj"] = "dan", ["dh"] = "dang", ["dk"] = "dao", ["de"] = "de", ["dz"] = "dei", ["df"] = "den", ["dg"] = "deng", ["di"] = "di", ["dw"] = "dia", ["dm"] = "dian", ["dc"] = "diao", ["dx"] = "die", ["dn"] = "din", ["d;"] = "ding", ["dq"] = "diu", ["ds"] = "dong", ["db"] = "dou", ["du"] = "du", ["dr"] = "duan", ["dv"] = "dui", ["dp"] = "dun", ["do"] = "duo", ["oe"] = "e", ["oz"] = "ei", ["of"] = "en", ["og"] = "eng", ["or"] = "er", ["fa"] = "fa", ["fj"] = "fan", ["fh"] = "fang", ["fz"] = "fei", ["ff"] = "fen", ["fg"] = "feng", ["fc"] = "fiao", ["fo"] = "fo", ["fs"] = "fong", ["fb"] = "fou", ["fu"] = "fu", ["ga"] = "ga", ["gl"] = "gai", ["gj"] = "gan", ["gh"] = "gang", ["gk"] = "gao", ["ge"] = "ge", ["gz"] = "gei", ["gf"] = "gen", ["gg"] = "geng", ["gs"] = "gong", ["gb"] = "gou", ["gu"] = "gu", ["gw"] = "gua", ["gy"] = "guai", ["gr"] = "guan", ["gd"] = "guang", ["gv"] = "gui", ["gp"] = "gun", ["go"] = "guo", ["ha"] = "ha", ["hl"] = "hai", ["hj"] = "han", ["hh"] = "hang", ["hk"] = "hao", ["he"] = "he", ["hz"] = "hei", ["hf"] = "hen", ["hg"] = "heng", ["hm"] = "hm",  ["hs"] = "hong", ["hb"] = "hou", ["hu"] = "hu", ["hw"] = "hua", ["hy"] = "huai", ["hr"] = "huan", ["hd"] = "huang", ["hv"] = "hui", ["hp"] = "hun", ["ho"] = "huo", ["ji"] = "ji", ["jw"] = "jia", ["jm"] = "jian", ["jd"] = "jiang", ["jc"] = "jiao", ["jx"] = "jie", ["jn"] = "jin", ["j;"] = "jing", ["js"] = "jiong", ["jq"] = "jiu", ["ju"] = "ju", ["jr"] = "juan", ["jt"] = "jue", ["jp"] = "jun", ["ka"] = "ka", ["kl"] = "kai", ["kj"] = "kan", ["kh"] = "kang", ["kk"] = "kao", ["ke"] = "ke", ["kz"] = "kei", ["kf"] = "ken", ["kg"] = "keng", ["ks"] = "kong", ["kb"] = "kou", ["ku"] = "ku", ["kw"] = "kua", ["ky"] = "kuai", ["kr"] = "kuan", ["kd"] = "kuang", ["kv"] = "kui", ["kp"] = "kun", ["ko"] = "kuo", ["la"] = "la", ["ll"] = "lai", ["lj"] = "lan", ["lh"] = "lang", ["lk"] = "lao", ["le"] = "le", ["lz"] = "lei", ["lg"] = "leng", ["li"] = "li", ["lw"] = "lia", ["lm"] = "lian", ["ld"] = "liang", ["lc"] = "liao", ["lx"] = "lie", ["ln"] = "lin", ["l;"] = "ling", ["lq"] = "liu", ["ls"] = "long", ["lb"] = "lou", ["lu"] = "lu", ["lr"] = "luan", ["lt"] = "lue", ["lp"] = "lun", ["lo"] = "luo", ["ly"] = "lv", ["ma"] = "ma", ["ml"] = "mai", ["mj"] = "man", ["mh"] = "mang", ["mk"] = "mao", ["me"] = "me", ["mz"] = "mei", ["mf"] = "men", ["mg"] = "meng", ["mi"] = "mi", ["mm"] = "mian", ["mc"] = "miao", ["mx"] = "mie", ["mn"] = "min", ["m;"] = "ming", ["mq"] = "miu", ["mo"] = "mo", ["mb"] = "mou", ["mu"] = "mu", ["na"] = "na", ["nl"] = "nai", ["nj"] = "nan", ["nh"] = "nang", ["nk"] = "nao", ["ne"] = "ne", ["nz"] = "nei", ["nf"] = "nen", ["ng"] = "neng", ["ni"] = "ni", ["nw"] = "nia", ["nm"] = "nian", ["nd"] = "niang", ["nc"] = "niao", ["nx"] = "nie", ["nn"] = "nin", ["n;"] = "ning", ["nq"] = "niu", ["ns"] = "nong", ["nb"] = "nou", ["nu"] = "nu", ["nr"] = "nuan", ["nt"] = "nue", ["np"] = "nun", ["no"] = "nuo", ["nv"] = "nv", ["oo"] = "o", ["ob"] = "ou", ["pa"] = "pa", ["pl"] = "pai", ["pj"] = "pan", ["ph"] = "pang", ["pk"] = "pao", ["pz"] = "pei", ["pf"] = "pen", ["pg"] = "peng", ["pi"] = "pi", ["pw"] = "pia", ["pm"] = "pian", ["pc"] = "piao", ["px"] = "pie", ["pn"] = "pin", ["p;"] = "ping", ["po"] = "po", ["pb"] = "pou", ["pu"] = "pu", ["qi"] = "qi", ["qw"] = "qia", ["qm"] = "qian", ["qd"] = "qiang", ["qc"] = "qiao", ["qx"] = "qie", ["qn"] = "qin", ["q;"] = "qing", ["qs"] = "qiong", ["qq"] = "qiu", ["qu"] = "qu", ["qr"] = "quan", ["qt"] = "que", ["qp"] = "qun", ["rj"] = "ran", ["rh"] = "rang", ["rk"] = "rao", ["re"] = "re", ["rf"] = "ren", ["rg"] = "reng", ["ri"] = "ri", ["rs"] = "rong", ["rb"] = "rou", ["ru"] = "ru", ["rw"] = "rua", ["rr"] = "ruan", ["rv"] = "rui", ["rp"] = "run", ["ro"] = "ruo", ["sa"] = "sa", ["sl"] = "sai", ["sj"] = "san", ["sh"] = "sang", ["sk"] = "sao", ["se"] = "se", ["sz"] = "sei", ["sf"] = "sen", ["sg"] = "seng", ["ua"] = "sha", ["ul"] = "shai", ["uj"] = "shan", ["uh"] = "shang", ["uk"] = "shao", ["ue"] = "she", ["uz"] = "shei", ["uf"] = "shen", ["ug"] = "sheng", ["ui"] = "shi", ["ub"] = "shou", ["uu"] = "shu", ["uw"] = "shua", ["uy"] = "shuai", ["ur"] = "shuan", ["ud"] = "shuang", ["uv"] = "shui", ["up"] = "shun", ["uo"] = "shuo", ["si"] = "si", ["ss"] = "song", ["sb"] = "sou", ["su"] = "su", ["sr"] = "suan", ["sv"] = "sui", ["sp"] = "sun", ["so"] = "suo", ["ta"] = "ta", ["tl"] = "tai", ["tj"] = "tan", ["th"] = "tang", ["tk"] = "tao", ["te"] = "te", ["tz"] = "tei", ["tg"] = "teng", ["ti"] = "ti", ["tm"] = "tian", ["tc"] = "tiao", ["tx"] = "tie", ["t;"] = "ting", ["ts"] = "tong", ["tb"] = "tou", ["tu"] = "tu", ["tr"] = "tuan", ["tv"] = "tui", ["tp"] = "tun", ["to"] = "tuo", ["wa"] = "wa", ["wl"] = "wai", ["wj"] = "wan", ["wh"] = "wang", ["wz"] = "wei", ["wf"] = "wen", ["wg"] = "weng", ["wo"] = "wo", ["ws"] = "wong", ["wu"] = "wu", ["xi"] = "xi", ["xw"] = "xia", ["xm"] = "xian", ["xd"] = "xiang", ["xc"] = "xiao", ["xx"] = "xie", ["xn"] = "xin", ["x;"] = "xing", ["xs"] = "xiong", ["xq"] = "xiu", ["xu"] = "xu", ["xr"] = "xuan", ["xt"] = "xue", ["xp"] = "xun", ["ya"] = "ya", ["yl"] = "yai", ["yj"] = "yan", ["yh"] = "yang", ["yk"] = "yao", ["ye"] = "ye", ["yi"] = "yi", ["yn"] = "yin", ["y;"] = "ying", ["yo"] = "yo", ["ys"] = "yong", ["yb"] = "you", ["yu"] = "yu", ["yr"] = "yuan", ["yt"] = "yue", ["yp"] = "yun", ["za"] = "za", ["zl"] = "zai", ["zj"] = "zan", ["zh"] = "zang", ["zk"] = "zao", ["ze"] = "ze", ["zz"] = "zei", ["zf"] = "zen", ["zg"] = "zeng", ["va"] = "zha", ["vl"] = "zhai", ["vj"] = "zhan", ["vh"] = "zhang", ["vk"] = "zhao", ["ve"] = "zhe", ["vz"] = "zhei", ["vf"] = "zhen", ["vg"] = "zheng", ["vi"] = "zhi", ["vs"] = "zhong", ["vb"] = "zhou", ["vu"] = "zhu", ["vw"] = "zhua", ["vy"] = "zhuai", ["vr"] = "zhuan", ["vd"] = "zhuang", ["vv"] = "zhui", ["vp"] = "zhun", ["vo"] = "zhuo", ["zi"] = "zi", ["zs"] = "zong", ["zb"] = "zou", ["zu"] = "zu", ["zr"] = "zuan", ["zv"] = "zui", ["zp"] = "zun", ["zo"] = "zuo" }

local function mspy_2_qp(input)
  local result_table = {}
  for i = 1, #input, 2 do
    local pair = input:sub(i, i + 1)
    if i + 1 > #input then
      pair = input:sub(i)
    end
    table.insert(result_table, mspy2qp_table[pair] or pair)
  end
  return result_table
end

local function processor(key, env)
  local context = env.engine.context
  if key:repr() == "Shift+Control+C" and context:is_composing() then
    flag = true
    context:refresh_non_confirmed_composition()
    return 1
  end
  return 2
end

local translator = {}

---@param env Env
function translator.init(env)
  local config = env.engine.schema.config
  local db_name = config:get_string("cloud_pinyin/reverse_db")
  if db_name then
    env.db = ReverseLookup(db_name)
  end
  local list = config:get_list("cloud_pinyin/providers")
  local providers = {}
  if list and list.size > 0 then
    for i = 0, list.size - 1 do
      table.insert(providers, list:get_value_at(i).value)
    end
  end
  local post_processing = config:get_string("cloud_pinyin/post_processing")
  env.suggest = helper.suggest.new()
  env.suggest.providers = providers or {"baidu"}
  env.suggest.post_processing = post_processing or "truncate"
  env.memory = Memory(env.engine, env.engine.schema)
  env.notifier = env.engine.context.commit_notifier:connect(function(ctx)
    local commit = ctx.commit_history:back()
    if commit and commit.type:sub(1, 6) == "cloud:" then
      local code = commit.type:sub(7)
      local text = commit.text
      -- 反查，按字查拼音，只有查询结果与输入拼音完全匹配时才加入词库
      if not env.db then
        return
      end
      if not helper.pinyin_match(text, code, function(c) return env.db:lookup(c) end) then
        return
      end
      local entry = DictEntry()
      entry.text = text
      entry.custom_code = code .. " "
      env.memory:start_session()
      local r = env.memory:update_userdict(entry, 1, "")
      env.memory:finish_session()
      --log.error(string.format("添加用户词典：%s, %s, %q", code, text, r))
    end
  end)
end

function translator.fini(env)
  env.notifier:disconnect()
  env.memory:disconnect()
  env.memory = nil
  env.db = nil
  env.suggest = nil
  collectgarbage()
end

---@param input string
---@param seg Segment
---@param env Env
function translator.func(input, seg, env)
  if not flag then
    return
  end
  flag = false
  local qp = mspy_2_qp(input)
  local code = table.concat(qp, " ")
  local reply = env.suggest:call(qp)
  for _, value in ipairs(reply) do
    local c = Candidate("cloud:" .. code, seg.start, seg._end, value, "☁️")
    c.quality = 2
    c.preedit = code
    yield(c)
  end
end

return { processor = processor, translator = translator }
