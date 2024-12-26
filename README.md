## 单字模式

![单字模式](./img/单字模式.gif)

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
