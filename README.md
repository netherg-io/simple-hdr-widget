# Simple HDR Widget

[![Tauri](https://img.shields.io/badge/Tauri-FFC131?style=flat-square&logo=tauri&logoColor=black)](https://tauri.app)
[![Nuxt](https://img.shields.io/badge/Nuxt-00DC82?style=flat-square&logo=nuxt.js&logoColor=white)](https://nuxt.com)
[![GitHub](https://img.shields.io/badge/GitHub-netherguy4-181717?style=flat-square&logo=github)](https://github.com/netherguy4)

A lightweight, modern HDR widget application built with Tauri and Vue 3.

## Features

- **Fast & Lightweight** - Built with Tauri for minimal resource usage
- **Modern UI** - Vue 3 with Nuxt framework
- **HDR Support** - Display and manage HDR content
- **Windows-Only** - Optimized for Windows platform
- **Responsive Design** - Beautiful UI with SCSS styling

## What is HDR?

HDR (High Dynamic Range) is a technology that displays a wider range of colors and brightness levels compared to standard SDR (Standard Dynamic Range) displays. HDR content provides:

- **Greater brightness range** - From deeper blacks to brighter whites
- **More colors** - Wider color gamut for more vibrant and accurate colors
- **Better contrast** - Improved detail in both dark and bright areas

⚠️ **Important:** HDR features require a compatible HDR-capable monitor or display. Without HDR support, the HDR toggle will have no visible effect.

## Quick Start

### Download Latest Release

The easiest way to get started:

1. Go to https://github.com/netherg-io/simple-hdr-widget/releases/latest
2. Download either:
   - **Installer** (`.msi`) - Recommended for most users
   - **Portable** (`.exe`) - Run without installation
3. Run the downloaded file and start using the widget

**Note:** This application only works on Windows.

## Prerequisites (For Development)

- Node.js 18+ and npm/yarn
- Rust 1.70+ (for Tauri)
- Python 3.7+ (build requirement)
- Windows OS

## Development Installation

1. Clone the repository:

```bash
git clone https://github.com/netherg-io/simple-hdr-widget
cd simple-hdr-widget
```

2. Install dependencies:

```bash
npm install
```

3. Start development server:

```bash
npm run dev
```

4. Build for production:

```bash
npm run build
```

## Configuration

The project uses Nuxt 3 with the following key configurations:

- **SSR**: Disabled (desktop app)
- **CSS**: Reset CSS + SCSS with utility imports
- **Modules**: VueUse, SVGO, ESLint
- **Vite**: Optimized for Tauri with environment prefix support

## Development

### Available Scripts

- `npm run dev` - Start dev server
- `npm run build` - Build for production
- `npm run lint` - Run ESLint
- `npm run preview` - Preview production build

## Building

To build the desktop application:

```bash
npm run tauri build
```

This will create platform-specific executables in `src-tauri/target/release/bundle/`.

## Troubleshooting

- **Port conflicts**: The dev server runs on port 5173 by default
- **Tauri issues**: Ensure Rust is properly installed with `rustup update`
- **Dependencies**: Clear node_modules and reinstall if experiencing issues
- **Windows-only**: This widget is not compatible with macOS or Linux
- **HDR not working**: Ensure your monitor supports HDR and it's enabled in Windows display settings

## Support

For issues and feature requests, please open an issue on the repository.
