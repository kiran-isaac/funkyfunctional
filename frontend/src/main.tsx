import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'

import init from 'sfl_wasm_lib';
import * as wasm from 'sfl_wasm_lib';

init().then(() => {
  wasm.my_init();
  createRoot(document.getElementById('root')!).render(
    <App />,
  )
});
