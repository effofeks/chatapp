# Setup instructions

### Step 1: Install Rust

Run the following command in your terminal:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions to install Rust and its related tools.

### Step 2: Set Up Your Environment

After installing Rust, you need to ensure that Rust and Cargo are accessible from your terminal. This requires verifying that the Rust binaries have been added to your system's `PATH`.


To check if Rust has been added to your system's `PATH`.

```bash
echo $PATH
```

If the output includes something like `"$HOME/.cargo/bin"`, it means Rust is already in your `PATH`. If the installer did not automatically add Rust to your `PATH`, you will need to do it manually. 

1. Check which shell you're using.

```bash
echo $SHELL
```

This will output the shell you're currently using, such as `/bin/bash` (Bash) or `/bin/zsh` (Zsh).

2. Modify your shell configuration file. 

- For example, if you're using **Zsh**, modify the `~/.zshrc` file:
     
     ```bash
     nano ~/.zshrc
     ```

- Add Rust to your `PATH` by appending the following line to your shell configuration file. 

    ```bash
    export PATH="$HOME/.cargo/bin:$PATH"
    ```

3. Apply the changes.

    ```bash
    source ~/.zshrc
    ```

Verify the installation by checking the Rust and Cargo versions.

```bash
rustc --version
cargo --version
```


### Step 3: Running the application

1. Clone the chatapp Rust repository:

```bash
git clone git@github.com:effofeks/chatapp.git
cd chatapp
```

2. Use `cargo` to build and run the Rust application:

```bash
cargo build
cargo run
```
