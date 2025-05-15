# DeepSeek CLI  

For Chinese, see [README.md](README.md)  

DeepSeek CLI is a command-line tool that enables multi-turn conversations by calling the DeepSeek API, with support for managing conversation history. ~~A clumsy imitation of Professor JYY's work.~~  

You can start or continue conversations in memory mode or use memoryless mode for single-turn interactions.  

## Features  

- **Memoryless Mode**: Directly input your query or use the `nomemory` command to enable this mode. The program only sends the current input to the API without loading or saving history.  
- **Memory Mode**: Activated using the `new` or `continue` commands:  
  - `new`: Starts a new conversation.  
  - `continue`: Resumes the most recent conversation from memory mode.  
  - Conversation history (only in memory mode) is stored in `~/.config/deepseek/histories/`.  
- ANSI color-coded prompts for quick identification of success/error messages.  
- Conversation history is saved in timestamp-based files for easier management.  
- Supports data transmission via pipes.  

## Installation  

1. Clone the repository:  

   ```sh  
   git clone <repository_url>  
   cd deepseek_cli  
   ```  

2. Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.  

3. Build the project:  

   ```sh  
   cargo build --release  
   ```  

4. The executable will be generated in `target/release/deepseek_cli`.  

## Configuration  

Below are example steps for macOS, Linux, and Windows to add the executable to your PATH for global access:  

<details>  
  <summary><strong>macOS</strong></summary>  

  Run the following command in the terminal (requires admin privileges) to symlink the executable to `/usr/local/bin` (usually already in PATH):  

  ```bash  
  sudo ln -s $(pwd)/target/release/deepseek_cli /usr/local/bin/ag  
  ```  
</details>  

<details>  
  <summary><strong>Linux</strong></summary>  

  Symlink method:  

  ```bash  
  sudo ln -s $(pwd)/target/release/deepseek_cli /usr/local/bin/ag  
  ```  

  Or copy the file:  

  ```bash  
  sudo cp $(pwd)/target/release/deepseek_cli /usr/local/bin/ag  
  ```  
</details>  

<details>  
  <summary><strong>Windows</strong></summary>  

  Run the following in an elevated Command Prompt or PowerShell:  

  ```cmd  
  copy target\release\deepseek_cli.exe C:\Windows\System32\ag.exe  
  ```  
</details>  

Ensure you have sufficient permissions to create symlinks or copy files in the target directory. After completing these steps, you can use the `ag` command globally in your terminal.  

Create a configuration file `config.toml` in `~/.config/deepseek/` containing your DeepSeek API key, e.g.:  

```toml  
api_key = "your_api_key_here"  
```  

Alternatively, use the command:  

```sh  
ag set_api your_api_key  
```  

This will generate the configuration file at `~/.config/deepseek/config.toml` by default.  

## Usage  

### Memoryless Mode  

Directly input your query:  

```sh  
ag "This is a memoryless query"  
```  

Or enable memoryless mode with the `nomemory` command:  

```sh  
ag nomemory "This is a memoryless query"  
```  

### Memory Mode  

Enable memory mode using the `new` or `continue` commands.  

- **New Conversation** (clears history):  

  ```sh  
  ag new "Is this our first conversation?"  
  ```  

- **Continue Conversation**:  

  Use the `continue` command to resume:  

  ```sh  
  ag continue "Let's continue our conversation."  
  ```  

The program automatically manages conversation history and displays prompts in memory mode (green for successful saves, yellow for no history retention).  

### Additional Parameters  

```sh  
Use the DeepSeek API for multi-turn conversations and manage conversation history  

Usage: ag [OPTIONS] [query] [COMMAND]  

Commands:  
  new       Start a new conversation  
  continue  Continue the last conversation  
  nomemory  Memoryless mode  
  set_api   Set API Key  
  help      Print this message or the help of the given subcommand(s)  

Arguments:  
  [query]  Query content  

Options:  
  -v, --version <version>          Model version, r1 for deepseek-reasoner [default: v3]  
  -t, --temperature <temperature>  Temperature (default: 1.0, range: 0.0-2.0, higher = more random) [default: 1.0]  
  -h, --help                      Print help  
  -V, --version                   Print version  
```  

## Examples  

```sh  
# Memoryless mode:  
ag "Hello, DeepSeek!"  

# Start a new conversation in memory mode:  
ag new "Hello, is this our first conversation?"  

# Continue a conversation in memory mode:  
ag "Please continue the previous topic."  
```  

## Development  

The project uses [Tokio](https://docs.rs/tokio) for async processing, [clap](https://docs.rs/clap) for command-line argument parsing, and [Reqwest](https://docs.rs/reqwest) to call the DeepSeek API.  

You can explore the modules in an IDE:  
- `src/main.rs` – Main logic and argument parsing  
- `src/history.rs` – Conversation history management  
- `src/config.rs` – Configuration handling  
- `src/models.rs` – Request/response data structures  

## License  

This project is licensed under the [MIT License](LICENSE).  

## Contact  

For questions or suggestions, contact the project maintainer or submit an issue.