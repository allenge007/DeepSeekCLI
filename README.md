# DeepSeek CLI

DeepSeek CLI 是一个命令行工具，通过调用 DeepSeek API 实现多轮对话，并支持对话历史记录的管理。~~是对jyy老师的拙劣模仿。~~

你可以在有记忆模式下新建/继续对话，也可以使用无记忆模式单次对话。

## 特性

- **无记忆模式**：直接输入对话，程序仅使用当前输入向 API 提问，不加载或保存历史记录。
- **记忆模式**：使用 `--memory` 参数启用，支持以下子命令：
  - `new`：新对话，清空历史记录并启动一个新的会话。
  - `continue`：继续上一次对话，自动加载保存的历史内容。
- ANSI 色彩提示，帮助你快速识别成功/错误信息。
- 对话历史记录以基于时间戳的文件保存，管理历史记录更加方便。
- 支持通过管道进行数据传输。

## 安装

1. 克隆代码仓库：

   ```sh
   git clone <仓库地址>
   cd deepseek_cli
   ```

2. 确保你已经安装了 [Rust](https://www.rust-lang.org/tools/install)。

3. 编译项目：

   ```sh
   cargo build --release
   ```

4. 可执行文件将在 `target/release/deepseek_cli` 中生成。

## 配置

在 `~/.config/deepseek/` 下创建配置文件 `config.toml`，确保包含你的 DeepSeek API key，例如：

```toml
api_key = "your_api_key_here"
```

下面是针对 macOS、Linux 和 Windows 系统的示例步骤，帮助你将生成的可执行文件放入 PATH 中，从而实现全局调用：

<details>
  <summary><strong>macOS</strong></summary>

  在终端中执行以下命令（需要管理员权限），将可执行文件链接到 `/usr/local/bin` 目录（通常已在 PATH 中）：

  ```bash
  sudo ln -s $(pwd)/target/release/deepseek_cli /usr/local/bin/ag
  ```
</details>

<details>
  <summary><strong>Linux</strong></summary>

  使用链接方式：
  
  ```bash
  sudo ln -s $(pwd)/target/release/deepseek_cli /usr/local/bin/ag
  ```

  或复制文件：
  
  ```bash
  sudo cp $(pwd)/target/release/deepseek_cli /usr/local/bin/ag
  ```
</details>

<details>
  <summary><strong>Windows</strong></summary>

  在管理员权限下的命令提示符或 PowerShell 中执行：
  
  ```cmd
  copy target\release\deepseek_cli.exe C:\Windows\System32\ag.exe
  ```
</details>

请确保你有足够的权限在对应目录中创建符号链接或复制文件，完成上述操作后，就可以在终端中直接使用 `ag` 命令全局调用该程序。



## 使用方法

### 无记忆模式

直接调用命令并输入对话内容：

```sh
./deepseek_cli "这是一条无记忆对话"
```

### 记忆模式

通过 `-m` 或 `--memory` 参数启用记忆模式。

- **新对话**（清空历史记录）:

  ```sh
  ./deepseek_cli -m new "这是我们的第一次对话吗"
  ```

- **继续对话**:
  
  指定 `continue` 子命令或直接使用 `-m` 参数后输入对话内容：

  ```sh
  ./deepseek_cli -m continue "继续我们的对话"
  # 或者
  ./deepseek_cli -m "继续我们的对话"
  ```

程序会自动管理对话历史记录，并在记忆模式下显示相应的提示信息（绿色表示成功保存，黄色表示不保存历史记录）。

## 示例

```sh
# 无记忆模式：
ag "你好，DeepSeek！"

# 开启记忆模式新对话：
ag -m new "你好，这是我们的第一次对话吗？"

# 开启记忆模式继续对话：
ag -m "请继续刚才的话题。"
```

## 开发

项目使用 [Tokio](https://docs.rs/tokio) 进行异步处理，使用 [clap](https://docs.rs/clap) 解析命令行参数，并通过 [Reqwest](https://docs.rs/reqwest) 调用 DeepSeek API。

你可以直接在 IDE 中打开项目，查看各模块文件：
- `src/main.rs` – 主逻辑及参数解析
- `src/history.rs` – 对话历史管理
- `src/config.rs` – 配置读取
- `src/models.rs` – 请求/响应数据结构

## 许可证

本项目遵循 [MIT License](LICENSE)。

## 联系方式

如有疑问或建议，请联系项目负责人或通过 issue 反馈。
