# @libvt/node

High-performance terminal emulation library for Node.js, powered by Rust.

## Features

- 🚀 **High Performance**: Native Rust implementation with zero-copy buffer access
- 🎨 **Full VT100/VT220/xterm Compatibility**: Complete escape sequence support
- 🌈 **256-color and True Color**: Full color palette support
- 🖱️ **Mouse Protocol Support**: Handle mouse events in terminal
- 🔗 **OSC 8 Hyperlinks**: Clickable links in terminal output
- 📊 **Unicode Support**: Proper width calculation for CJK and emoji
- 🎯 **TypeScript Support**: Full TypeScript type definitions included

## Installation

```bash
npm install @libvt/node
```

## Quick Start

```javascript
const { Terminal } = require('@libvt/node');

// Create an 80x24 terminal
const term = new Terminal(80, 24);

// Write some text
term.write(Buffer.from('Hello, World!\r\n'));

// Write ANSI escape sequences
term.write(Buffer.from('\x1b[1;31mRed Bold Text\x1b[0m\r\n'));

// Get cursor position
const { col, row } = term.cursorPosition();
console.log(`Cursor at: (${col}, ${row})`);

// Get visible text
const text = term.getVisibleText();
console.log(text);
```

## TypeScript Example

```typescript
import { Terminal, TerminalConfig } from '@libvt/node';

const config: TerminalConfig = {
  scrollbackLines: 10000,
  enableImages: true,
  enableHyperlinks: true,
  bracketedPaste: true,
};

const term = new Terminal(120, 40, config);

term.write(Buffer.from('TypeScript Terminal!\n'));

const cell = term.getCell(0, 0);
if (cell) {
  console.log(`Character: ${cell.text}`);
  console.log(`Bold: ${cell.attrs.bold}`);
  console.log(`FG Color: #${cell.fgColor.toString(16)}`);
}
```

## API Documentation

### Terminal Class

#### Constructor

```typescript
new Terminal(cols: number, rows: number, config?: TerminalConfig)
```

Creates a new terminal instance.

#### Methods

- `write(data: Buffer | Uint8Array): void` - Write data to the terminal
- `cursorPosition(): CursorPosition` - Get current cursor position
- `size(): CursorPosition` - Get terminal size
- `resize(cols: number, rows: number): void` - Resize the terminal
- `getCell(col: number, row: number): Cell | null` - Get cell at position
- `getLineText(row: number): string | null` - Get text from a line
- `getVisibleText(): string` - Get all visible text
- `clear(): void` - Clear the screen
- `reset(): void` - Reset terminal to initial state
- `encodeKey(key: string, ctrl: boolean, alt: boolean, shift: boolean): Buffer` - Encode a key event
- `isLineDirty(row: number): boolean` - Check if line is dirty
- `clearDirty(): void` - Clear dirty line tracking

### Functions

- `version(): string` - Get library version
- `createTerminal(cols: number, rows: number): Terminal` - Create terminal with default config

## Performance

libvt-node is designed for high performance:

- **Zero-copy buffer access**: Direct access to terminal buffer without copying
- **Native Rust implementation**: ~10x faster than pure JavaScript implementations
- **Efficient dirty tracking**: Only re-render changed lines
- **SIMD optimizations**: Vectorized operations where applicable

## Compatibility

- **Node.js**: >= 16.0.0
- **Platforms**: Linux (x64, ARM64), macOS (x64, Apple Silicon), Windows (x64, ARM64)
- **Architectures**: x86_64, aarch64, armv7

## License

MIT

## Contributing

Contributions are welcome! Please see the main WezTerm repository.
