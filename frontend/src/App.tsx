import { useState } from 'react'
import Input from './Input'
import * as wasm from 'sfl_wasm_lib'
import './App.css'
import RC from './RC';
import { DefinitionWindow, DefinitionSpawnButton } from './help';

function App() {
  const [astString, setAstString] = useState("");
  const [rcs, setRcs] = useState<JSX.Element[]>([]);
  const [errorString, setErrorString] = useState("");
  const [definitionIsVisible, setDefinitionIsVisible] = useState(true);

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
        ast = wasm.pick_rc_and_free(ast, rcs, rc_index);
        setAstString(wasm.to_string(ast));
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
      setAstString("")
      setRcs([])
    }
  }

  const handleRun = (programInput: string, multiple: boolean) => {
    try {
      const ast = wasm.parse(programInput);
      setAstString(wasm.to_string(ast))
      generateRCs(ast, multiple);

      setErrorString("")
    } catch (e) {
      setErrorString(e as string)
      setAstString("")
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
          <pre>{astString}</pre>
        </div>
        <div id="Error">
          <pre>{errorString}</pre>
        </div>
        <ul id="RCArea">
          {rcs}
        </ul>
      </div>
    </>
  )
}

export default App
