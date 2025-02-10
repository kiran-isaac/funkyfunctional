import { useState } from 'react'
import Input from './Input'
import * as wasm from 'sfl_wasm_lib'
import './App.css'
import RC from './RC';
import { DefinitionWindow, DefinitionSpawnButton } from './help';
import ASTHistory from './ASTHistory';

function App() {
  const [typeAssignsString, setTypeAssignsToString] = useState("");
  const [originalExprString, setOriginalExprString] = useState("");
  const [rcs, setRcs] = useState<JSX.Element[]>([]);
  const [selectedRcFromStringHistory, setSelectedRcFromStringHistory] = useState<string[]>([]);
  const [selectedRcToStringHistory, setSelectedRcToStringHistory] = useState<string[]>([]);
  const [errorString, setErrorString] = useState("");
  const [definitionIsVisible, setDefinitionIsVisible] = useState(true);
  const [astHistory, setAstHistory] = useState<wasm.RawASTInfo[]>([]);

  const generateRCs = (ast: wasm.RawASTInfo, multiple: boolean) => {
    try {
      const rcs = multiple ? wasm.get_all_redexes(ast) : wasm.get_one_redex(ast);

      if (wasm.get_rcs_len(rcs) === 0) {
        setRcs([]);
        return;
      }

      const rc_callback = (rc_index: number) => {
        const from_string = wasm.get_rcs_from(rcs, rc_index);
        const to_string = wasm.get_rcs_to(rcs, rc_index);
        console.log(from_string, to_string);

        // add the current ast to the history
        setAstHistory((prevAstHistory) => {
          const newAstHistory = [...prevAstHistory, ast];
          return newAstHistory;
        });
        setSelectedRcFromStringHistory((prev) => {
          const newRcFromStringHistory = [...prev, from_string];
          return newRcFromStringHistory;
        });
        setSelectedRcToStringHistory((prev) => {
          const newRcToString = [...prev, to_string];
          return newRcToString;
        });
        ast = wasm.pick_rc_and_free(ast, rcs, rc_index);
        setOriginalExprString(wasm.main_to_string(ast));
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
      console.log(e);
      setErrorString(e as string)
      setRcs([])
      setAstHistory([])
      setOriginalExprString("")
      setSelectedRcFromStringHistory([])
      setSelectedRcToStringHistory([])
      setTypeAssignsToString("")
    }
  }

  const handleRun = (programInput: string, multiple: boolean) => {
    try {
      const ast = wasm.parse(programInput);
      setAstHistory([ast]);
      setOriginalExprString(wasm.main_to_string(ast));
      setSelectedRcFromStringHistory([]);
      setSelectedRcToStringHistory([]);
      setTypeAssignsToString(wasm.types_to_string(ast));
      generateRCs(ast, multiple);
      
      setErrorString("")
    } catch (e) {
      setErrorString(e as string)
      setRcs([])
      setAstHistory([])
      setOriginalExprString("")
      setSelectedRcFromStringHistory([])
      setSelectedRcToStringHistory([])
      setTypeAssignsToString("")
    };
  };

  return (
    <>
      <DefinitionSpawnButton definitionIsVisible={definitionIsVisible} setDefinitionIsVisible={setDefinitionIsVisible} />
      <DefinitionWindow definitionIsVisible={definitionIsVisible} setDefinitionIsVisible={setDefinitionIsVisible} />
      <div id="inputContainer">
        <Input
          onRunMultiple={(editorValue) => handleRun(editorValue, true)}
          onRunSingle={(editorValue) => handleRun(editorValue, false)}
        />
      </div>
      <div id="Spacer"></div>
      <div id="TextArea">
        <div id="ASTArea">
          {typeAssignsString && (
            <>
              <h4>Types:</h4>
              <pre>{typeAssignsString}</pre>
            </>
          )}

          <ul id="RCArea">
            {rcs}
          </ul>

          {originalExprString && (
            <>
              <h4>Main Expression:</h4>
              <pre>{originalExprString}</pre>
              <hr></hr>
            </>
          )}
        </div>
        <div id="Error">
          <pre>{errorString}</pre>
        </div>
        <pre><ASTHistory rcFromHistory={selectedRcFromStringHistory} rcToHistory={selectedRcToStringHistory} astHistory={astHistory} /></pre>
      </div>
    </>
  )
}

export default App
