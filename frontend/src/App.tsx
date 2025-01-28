import { useState } from 'react'
import Input from './Input'
import * as wasm from 'sfl_wasm_lib'
import './App.css'
import RC from './RC';
import { DefinitionWindow, DefinitionSpawnButton } from './help';
import ASTHistory from './ASTHistory';

function App() {
  const [originalAstString, setOriginalAstString] = useState("");
  const [rcs, setRcs] = useState<JSX.Element[]>([]);
  const [errorString, setErrorString] = useState("");
  const [definitionIsVisible, setDefinitionIsVisible] = useState(true);
  const [astHistory, setAstHistory] = useState<wasm.RawASTInfo[]>([]);

  const generateRCs = (ast: wasm.RawASTInfo, multiple: boolean) => {
    try {
      const rcs = multiple ? wasm.get_all_redexes(ast) : wasm.get_one_redex(ast);

      if (wasm.get_rcs_len(rcs) === 0) {
        return;
      }

      if (!multiple && wasm.get_rcs_len(rcs) > 1) {
        alert("WTF")
      }

      const rc_callback = (rc_index: number) => {
        // add the current ast to the history
        // add the current ast to the history
        setAstHistory((prevAstHistory) => {
          const newAstHistory = [...prevAstHistory, ast];
          return newAstHistory;
        });
        ast = wasm.pick_rc_and_free(ast, rcs, rc_index);
        generateRCs(ast, multiple);
      };

      const rc_elems = [];
      for (let i = 0; i < wasm.get_rcs_len(rcs); i++) {
        const from_string = wasm.get_rcs_from(rcs, i);
        const to_string = wasm.get_rcs_to(rcs, i);
        rc_elems.push(<RC key={i + 1} i={i} onClick={rc_callback} from={from_string} to={to_string} />);
      }

      setRcs(rc_elems);
    } catch (e) {
      setErrorString(e as string)
      setRcs([])
    }
  }

  const handleRun = (programInput: string, multiple: boolean) => {
    try {
      const ast = wasm.parse(programInput);
      setAstHistory([]);
      setOriginalAstString(wasm.to_string(ast));
      generateRCs(ast, multiple);

      setErrorString("")
    } catch (e) {
      setErrorString(e as string)
      setRcs([])
    };
  };

  return (
    <>
      <DefinitionSpawnButton definitionIsVisible={definitionIsVisible} setDefinitionIsVisible={setDefinitionIsVisible} />
      <DefinitionWindow definitionIsVisible={definitionIsVisible} setDefinitionIsVisible={setDefinitionIsVisible} />
      <div id="inputContainer">
        <Input onRunMultiple={(editorValue) => handleRun(editorValue, true)} onRunSingle={(editorValue) => handleRun(editorValue, false)} />
      </div>
      <div id="Spacer"></div>
      <div id="TextArea">
        <div id="ASTArea">
          <pre>{originalAstString}</pre>
        </div>
        <div id="Error">
          <pre>{errorString}</pre>
        </div>
        <ul id="RCArea">
          {rcs}
        </ul>
        <pre><ASTHistory astHistory={astHistory} /></pre>
      </div>
    </>
  )
}

export default App
