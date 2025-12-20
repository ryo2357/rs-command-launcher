---
name: scrap
description: ここまでの会話を、スクラップとして記録します。
model: Raptor mini (Preview) (copilot)
tools: ['edit']
---


- ここまでの会話内容を`scrap`として記録してください。
- 会話内の重複内容の省略以外の省略・要約は不要。
- 最新のメッセージだけでなく、必要に応じて関連する過去のメッセージも含めてください。
- 複数の提案・手段が会話で挙がっていた場合、それらの違い・メリットデメリットは省略しないでください。
- 追加の意見や推測は`scrap`に含めないでください。

## Scrap specifications

- Scraps are written in markdown format in the `.docs/scraps` directory.
- Scrap filenames must follow these rules:
  - Use the format `YYYYMMDD-title.md`, including the date and the title.
  - The title should concisely express the idea in Japanese.
- Avoid using tables in markdown; use bulleted lists instead.
- Avoid using strong emphasis (e.g., bold or italics) in markdown.
- Code, commands, type/generic examples, and placeholders must be written as inline code, e.g., `ffmpeg ...` / `Arc<RwLock<T>>` / `command ["seek", "<SEC>", "absolute+exact"]`
- For multi-line code, use code blocks.