import { useEffect, useState } from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/vite.svg'
import './App.css'

import * as monaco from 'monaco-editor';

import Editor from '@monaco-editor/react';

function App() {
  useEffect(() => {
    // Access the document object and perform actions
    const editorContainer = document.getElementById('editor-container');
    if (editorContainer) {
      console.log('Editor container found:', editorContainer);
      const editor = monaco.editor.create(editorContainer, {
        value: 'console.log("Hello, world!")',
      });
      const editor_dom: HTMLElement | null = editor.getDomNode();

      if (editor_dom) {
        // Use editor_dom safely here
        editor_dom.style.width = '100%';
        editor_dom.style.height = '100%';
        editorContainer.appendChild(editor_dom);
      } else {
        console.error('Editor DOM element not found');
      }
    }
  }, []); // Empty dependency array ensures this runs only once after the component mounts

  return (
    <div id="editor-container" style={{ height: '500px' }}>
      {/* Your editor component or other content */}
    </div>
  );
}

// function App() {
  
//   const [count, setCount] = useState(0)

//   return (
//     <>
//       <div>
//         <a href="https://vite.dev" target="_blank">
//           <img src={viteLogo} className="logo" alt="Vite logo" />
//         </a>
//         <a href="https://react.dev" target="_blank">
//           <img src={reactLogo} className="logo react" alt="React logo" />
//         </a>
//       </div>
//       <h1>Vite + React</h1>
//       <div className="card">
//         <button onClick={() => setCount((count) => count + 1)}>
//           count is {count}
//         </button>
//         <p>
//           Edit <code>src/App.tsx</code> and save to test HMR
//         </p>
//       </div>
//       <p className="read-the-docs">
//         Click on the Vite and React logos to learn more
//       </p>
//     </>
//   )
// }

export default App
