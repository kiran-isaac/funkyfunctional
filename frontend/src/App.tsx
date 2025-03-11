import {useState} from 'react'
import Input from './Input'
import * as wasm from 'sfl_wasm_lib'
import './App.css'
import './rhs.css'
import RC from './RC';
import ASTHistory from './ASTHistory';
import Buttons from './Buttons'

function App() {
  const [rcs, setRcs] = useState<JSX.Element[]>([]);
  const [editorValue, setEditorValue] = useState("");
  const [selectedRcFromStringHistory, setSelectedRcFromStringHistory] = useState<string[]>([]);
  const [selectedRcToStringHistory, setSelectedRcToStringHistory] = useState<string[]>([]);
  const [errorString, setErrorString] = useState("");
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
          return [...prevAstHistory, ast];
        });
        setSelectedRcFromStringHistory((prev) => {
          return [...prev, from_string];
        });
        setSelectedRcToStringHistory((prev) => {
          return [...prev, to_string];
        });
        ast = wasm.pick_rc_and_free(ast, rcs, rc_index);
        generateRCs(ast, multiple);
      };

      const rc_elems = [];
      for (let i = 0; i < wasm.get_rcs_len(rcs); i++) {
        const from_string = wasm.get_rcs_from(rcs, i);
        const to_string = wasm.get_rcs_to(rcs, i);
        rc_elems.push(<RC multiple={multiple} key={i + 1} i={i} onClick={rc_callback} from={from_string} to={to_string} />);
      }

      setRcs(rc_elems);
    } catch (e) {
      console.log(e);
      setErrorString(e as string)
      setRcs([])
      setAstHistory([])
      setSelectedRcFromStringHistory([])
      setSelectedRcToStringHistory([])
    }
  }

  const handleRun = (programInput: string, multiple: boolean) => {
    try {
      const ast = wasm.parse(programInput);
      setAstHistory([ast]);
      setSelectedRcFromStringHistory([]);
      setSelectedRcToStringHistory([]);
      generateRCs(ast, multiple);
      
      setErrorString("")
    } catch (e) {
      setErrorString(e as string)
      setRcs([])
      setAstHistory([])
      setSelectedRcFromStringHistory([])
      setSelectedRcToStringHistory([])
    }
  };

  return (
    <>
      <div id="lhs">
        <div id="Title">
          <div id="TitleFlex">
            <h1>SFL Explorer</h1>
            <p> by </p>
            <a href='https://github.com/kiran-isaac' target='blank'>Kiran Sturt</a>
          </div>
        </div>
        <div id="inputContainer">
          <Input
            editorValue={editorValue}
            setEditorValue={setEditorValue}
          />
          <Buttons 
            handleRun={handleRun}
            setEditorValue={setEditorValue}
            editorValue={editorValue}
          />
        </div>
      </div>

      <div id="rhs">
        <div id="Spacer"></div>
        <div id="TextArea">
          <div id="ASTArea">
            <ul id="RCArea">
              {rcs}
            </ul>
          </div>
          <div id="Error">
            <p>{errorString}</p>
          </div>
          <ASTHistory rcFromHistory={selectedRcFromStringHistory} rcToHistory={selectedRcToStringHistory} astHistory={astHistory} />
        </div>
      </div>
    </>
  )
}

export default App
