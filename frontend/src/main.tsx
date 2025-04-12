import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import './registerSW.ts'

import init from 'sfl_wasm_lib';
import * as wasm from 'sfl_wasm_lib';
import { SettingsProvider } from './SettingsProvider.tsx';

init().then(() => {
  wasm.my_init();
  createRoot(document.getElementById('root')!).render(
    <SettingsProvider>
      <App />,
    </SettingsProvider >
  )
});
