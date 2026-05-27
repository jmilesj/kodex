<p align="center"><strong>Kodex CLI</strong> is a coding agent that runs locally on your computer.
<p align="center">
  <img src="https://github.com/openai/codex/blob/main/.github/codex-cli-splash.png" alt="Kodex CLI splash" width="80%" />
</p>
</br>
If you want Kodex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE.</a>
</br>If you want the desktop app experience, run <code>kodex app</code> or visit <a href="https://chatgpt.com/codex?app-landing-page=true">the Codex App page</a>.
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.</p>

---

## Quickstart

### Installing and running Kodex CLI

Run the following on Mac or Linux to install Codex CLI:

```shell
curl -fsSL https://raw.githubusercontent.com/jmilesj/kodex/main/scripts/install/install.sh | sh
```

Then simply run `kodex` to get started.

<details>
<summary>You can also go to the <a href="https://github.com/jmilesj/kodex/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains platform-specific binaries:

- macOS
  - Apple Silicon/arm64: `kodex-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `kodex-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `kodex-x86_64-unknown-linux-gnu.tar.gz`
  - arm64: `kodex-aarch64-unknown-linux-gnu.tar.gz`

</details>

### Using Kodex with your ChatGPT plan

Run `kodex` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Kodex as part of your Plus, Pro, Business, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](https://help.openai.com/en/articles/11369540-codex-in-chatgpt).

You can also use Kodex with an API key, but this requires [additional setup](https://developers.openai.com/codex/auth#sign-in-with-an-api-key).

## Docs

- [**Kodex Documentation**](https://developers.openai.com/codex)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Fork tracking**](./docs/fork-tracking/README.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
