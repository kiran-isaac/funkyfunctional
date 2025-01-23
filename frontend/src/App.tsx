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
    const rcs = wasm.get_redexes(ast);

    if (wasm.get_rcs_len(rcs) == 0) {
      setRcs([]);
      return;
    }

    const rc_callback = (rc_index: number) => {
      wasm.pick_rc_and_free(ast, rcs, rc_index);
      setAstString(wasm.to_string(ast));
      generateRCs(ast);
    };

    const laziest = wasm.get_laziest(ast, rcs);

    console.log(laziest);

    const rc_elems = [<div key={0}><button className="rc" onClick={() => rc_callback(laziest)}>Laziest</button><br/></div>];

    for (let i = 0; i < wasm.get_rcs_len(rcs); i++) {
      const from_string = wasm.get_rcs_from(rcs, i);
      const to_string = wasm.get_rcs_to(rcs, i);
      rc_elems.push(<RC key={i + 1} i={i} onClick={rc_callback} from={from_string} to={to_string} />);
    }

    setRcs(rc_elems);
  }

  const handleRun = (programInput: string) => {
    try {
      const ast = wasm.parse(programInput);
      setAstString(wasm.to_string(ast))
      generateRCs(ast);
      
      setErrorString("")
    } catch (e) {
      setErrorString(e as string)
      setAstString("")
      setRcs([])
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
