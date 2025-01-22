import { useState } from 'react'
import Input from './Input'
import * as wasm from 'sfl_wasm_lib'
import './App.css'

export function escapeHtml(unsafe: string): string {
  return unsafe
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

function App() {
  let ast = wasm.parse("main = 2");
  let ast_str = wasm.to_string(ast);

  return (
    <>
      <div id="inputContainer">
        <Input/>
      </div>
      <div id="Spacer"></div>
      <div id="RCArea">
        <pre>{escapeHtml(ast_str)}</pre>
      </div>
    </>
  )
}

export default App
