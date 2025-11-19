// Simple test for libvt-node bindings

console.log('Testing libvt-node...\n');

try {
  const { Terminal, version, createTerminal } = require('./index.js');

  // Test version
  console.log(`✓ Library version: ${version()}`);

  // Test terminal creation
  const term = new Terminal(80, 24);
  console.log('✓ Created terminal: 80x24');

  // Test writing
  term.write(Buffer.from('Hello, World!\r\n'));
  console.log('✓ Write: "Hello, World!"');

  // Test cursor position
  const pos = term.cursorPosition();
  console.log(`✓ Cursor position: (${pos.col}, ${pos.row})`);

  // Test ANSI escape sequences
  term.write(Buffer.from('\x1b[1;31mRed Bold Text\x1b[0m\r\n'));
  console.log('✓ Write ANSI sequences');

  // Test get cell
  const cell = term.getCell(0, 0);
  if (cell) {
    console.log(`✓ Get cell [0,0]: "${cell.text}" (fg: ${cell.fgColor.toString(16)}, bold: ${cell.attrs.bold})`);
  }

  // Test visible text
  const text = term.getVisibleText();
  console.log(`✓ Visible text (${text.length} chars)`);

  // Test resize
  term.resize(120, 40);
  const newSize = term.size();
  console.log(`✓ Resized to: ${newSize.col}x${newSize.row}`);

  // Test line text
  const lineText = term.getLineText(0);
  if (lineText !== null) {
    console.log(`✓ Line 0 text: "${lineText.trimEnd()}"`);
  }

  // Test dirty tracking
  term.clearDirty();
  term.write(Buffer.from('test'));
  const isDirty = term.isLineDirty(0);
  console.log(`✓ Dirty tracking: line 0 is ${isDirty ? 'dirty' : 'clean'}`);

  // Test clear selection
  term.clearSelection();
  console.log('✓ Clear selection');

  // Test clear markers
  term.clearMarkers();
  console.log('✓ Clear markers');

  // Test createTerminal helper
  const term2 = createTerminal(100, 30);
  console.log('✓ createTerminal helper');

  console.log('\n✅ All tests passed!');
} catch (error) {
  console.error('\n❌ Test failed:', error.message);
  console.error(error.stack);
  process.exit(1);
}
