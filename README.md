## 单字模式

![单字模式](./img/单字模式.gif)

* 按 0 切换单字模式
* 选字或退出输入状态后自动关闭单字模式
* 处于单字模式时，按 退格键 关闭单字模式

### 安装方法

1. 下载 [single_char_mode.lua](./lua/single_char_mode.lua)，并将其放入 `<Rime 用户文件夹>/lua/` 文件夹中。

2. 修改所用方案的 `xxx.schema.yaml` 配置文件，找到 `engine` 字段，在其中添加以下内容：

```yaml
engine:
  processors:
    - ascii_composer
    - recognizer
    - key_binder
    - speller
    - punctuator
    # 添加这行，一定要插入到 selector 之前
    - lua_processor@*single_char_mode*processor
    - selector
    # ... 省略
  filters:
    # ... 省略
    # 添加这行，在列表末尾加入
    - lua_filter@*single_char_mode*filter
```

也可使用 patch 模式，但不推荐，因为 processor 插入位置不好确定。在所用方案对应的 `xxx.custom.yaml` 文件中添加以下内容：

```yaml
patch:
  engine/filters/+:
    - lua_filter@*single_char_mode*filter
  # @before 6 可用于白霜、雾凇等大部分方案，其他方案按实际情况修改
  engine/processors/@before 6: lua_processor@*single_char_mode*processor
```

3. 重新部署后即可生效。

---

## 云输入

![云输入](./img/云输入.gif)

* 微软双拼，按 Ctrl+Shift+C 调用百度云输入
* 选词上屏后自动加入词库，下次就可直接输入
* 如果云结果的拼音与原始输入不一致，会在备注内显示实际拼音

### 安装方法

1. 下载 [cloud_pinyin_mspy.lua](./lua/cloud_pinyin_mspy.lua) 和 [json.lua](https://github.com/rxi/json.lua/blob/master/json.lua) 并将其放入 `<Rime 用户文件夹>/lua/` 文件夹中。
2. 从 <https://github.com/hchunhui/librime-cloud/releases> 下载 `simplehttp.dll`，并将其放入 `<Rime 安装文件夹>` 中，如 `C:\Program Files\Rime\weasel-0.16.3`。
3. 依赖 2024.05.19 之后的 librime-lua 插件，但 weasel 0.16.3 早于此版本，可从 <https://github.com/rime/weasel/releases/tag/latest> 下载最新的 weasel 每夜版。
4. 修改所用方案的 `xxx.custom.yaml` 配置文件，在其中添加以下内容：

```yaml
patch:
  engine/processors/+:
    - lua_processor@*cloud_pinyin_mspy*processor
  engine/translators/+:
    - lua_translator@*cloud_pinyin_mspy*translator
```

5. 重新部署后即可生效。

### 感谢

* https://github.com/hchunhui/librime-cloud/issues/14#issuecomment-2222450807
