import { useState } from 'react'
import Input from './Input'
import * as wasm from 'sfl_wasm_lib'
import './App.css'
import RC from './RC';

function App() {
  const [astString, setAstString] = useState("");
  const [rcs, setRcs] = useState<JSX.Element[]>([]);
  const [errorString, setErrorString] = useState("");

  const generateRCs = (ast: wasm.RawASTInfo) => {
    let rcs = wasm.get_redexes(ast);
    let rcs_strings = [];
    let i = 0;
    for (let rc of rcs) {
      i++;
      let callback = () => {
        wasm.pick_rc_and_free(ast, rcs, i - 1);
        setAstString(wasm.to_string(ast));
        generateRCs(ast);
      };
      let from_string = wasm.get_rc_from(rc);
      let to_string = wasm.get_rc_to(rc);
      rcs_strings.push(<><RC onClick={callback} from={from_string} to={to_string} i={i} /><br /></>);
    }
    setRcs(rcs_strings);
  }

  const handleRun = () => {
    const programInput = (document.getElementById("ProgramInput") as HTMLTextAreaElement).value;
    try {
      const ast = wasm.parse(programInput);
      setAstString(wasm.to_string(ast))
      generateRCs(ast);
      
      setErrorString("")
    } catch (e: any) {
      setErrorString(e.toString())
      setAstString("")
    };
  };

  return (
    <>
      <div id="inputContainer">
        <Input onRun={handleRun} />
      </div>
      <div id="Spacer"></div>
      <div id="TextArea">
        <div id="ASTArea">
          <pre>{astString}</pre>
        </div>
        <div id="Error">
          <pre>{errorString}</pre>
        </div>
        <div id="RCArea">
          <pre>{rcs}</pre>
        </div>
      </div>
    </>
  )
}

export default App
