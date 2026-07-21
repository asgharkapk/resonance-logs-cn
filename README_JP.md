# Resonance Logs CN

[Blue Protocol: Star Resonance](https://www.starresonance.com/) 向けの戦闘解析デスクトップアプリケーションです。リアルタイム DPS メーター、Buff 監視、スキルクールダウン表示などの機能を提供します。

本プロジェクトは [resonance-logs](https://github.com/resonance-logs/resonance-logs) をベースに改変されています。

## 主な機能

- **DPS メーター**：リアルタイム DPS、戦闘時間（アクティブ時間）、履歴の記録
- **Buff モニター**：スキルクールダウン、Buff 監視、Buff エイリアス、カテゴリ別クイック監視（料理／錬金術）
- **モジュール最適化**：パケットキャプチャから取得したモジュールデータを基に、最適なモジュール構成を自動計算（GPU アクセラレーション対応）
- **カスタムパネル**：カウンターや Buff の状態をプログレスバー形式で表示
- **ゲームオーバーレイ**：透過・常に最前面表示のオーバーレイ。マスク機能とホットキー切り替えに対応
- **自動アップデート**：アプリ内 OTA アップデート対応

## 技術スタック

- **バックエンド**：Rust + [Tauri 2](https://tauri.app/)
- **フロントエンド**：SvelteKit 5 + Svelte + TypeScript + Tailwind CSS
- **パケットキャプチャ**：WinDivert / Npcap（Windows ネットワークパケットキャプチャ）

## システム要件

- **対応プラットフォーム**：Windows（WinDivert は管理者権限が必要。Npcap は [Npcap](https://npcap.com/) をインストールし、正しいネットワークアダプターを選択してください）
- **Node.js**：フロントエンドのビルドに必要
- **Rust**：Tauri アプリケーションのビルドに必要

## クイックスタート

### 依存関係のインストール

```bash
npm install
```

### 開発モード

```bash
npm run tauri dev
```

### ビルド

```bash
npm run tauri build
```

ビルド成果物はデフォルトで `src-tauri/target/release/bundle/` に出力され、NSIS インストーラーが生成されます。

### モジュール最適化機能のビルド

モジュール最適化機能は C++ 拡張に依存し、GPU（CUDA/OpenCL）によるアクセラレーションを利用できます。ビルド前に、[StarResonanceAutoMod](https://github.com/fudiyangjin/StarResonanceAutoMod) の環境要件を参照してください。

**基本要件（CPU 版）：**

- **Visual Studio Build Tools 2019/2022** または Visual Studio（MSVC コンパイラ）
- Windows SDK

**GPU 版の追加要件：**

- **CUDA Toolkit 12.9**（NVIDIA RTX 20 シリーズ以降。CUDA 12.9 以上で RTX 50 シリーズのビルドに対応）
- または **OpenCL**（NVIDIA / AMD / Intel GPU。通常は GPU ドライバーまたは CUDA に含まれます）

ビルド時に CUDA/OpenCL は自動検出されます。検出されない場合は CPU 版のみがビルドされます。また、C++ ソースディレクトリ `src-tauri/src/module_optimizer/cpp/` が存在しない場合は、モジュール最適化機能のビルドはスキップされますが、その他の機能には影響しません。

## ドキュメント

ドキュメントは **簡体中文**、**English**、**日本語** の 3 言語に対応しています。

- ソース： [doc/zh-CN/](./doc/zh-CN/README.md) · [doc/en-US/](./doc/en-US/README.md) · [doc/ja-JP/](./doc/ja-JP/README.md)
- 初めて利用する場合は、各言語の [はじめに / Getting Started / 快速入门](./doc/ja-JP/getting-started.md) を先にお読みください（Npcap のインストール、ネットワークアダプターの選択、**アプリの再起動** を含みます）
- HTML ドキュメントの生成：`npm run doc:html` → 出力先： [doc/html_doc/](./doc/html_doc/index.html)（言語切り替え対応）

## ダウンロード

- [Releases](https://github.com/fudiyangjin/resonance-logs-cn/releases) - ビルド済みインストーラー

## コミュニティ

- QQ グループ：1084866292
- Discord：https://discord.gg/RHeX47wvDU

## 更新履歴

詳細は [CHANGELOG.md](./CHANGELOG.md) をご覧ください。

## ライセンス

[AGPL-3.0-only](LICENSE)

## 謝辞

- [resonance-logs](https://github.com/resonance-logs/resonance-logs) - オリジナルプロジェクト
- [BPSR-ZDPS](https://github.com/Blue-Protocol-Source/BPSR-ZDPS) - ZDPS プロジェクト（DPS メーターおよび関連機能の参考）
- [StarResonanceDamageCounter](https://github.com/dmlgzs/StarResonanceDamageCounter) - ダメージメーター実装の参考
- [StarResonanceAutoMod](https://github.com/fudiyangjin/StarResonanceAutoMod) - モジュール最適化アルゴリズムおよびビルド方法の参考
