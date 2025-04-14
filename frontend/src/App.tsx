import { useState } from 'react'
import Input from './Input'
import * as wasm from 'sfl_wasm_lib'
import './App.css'
import './rhs.css'
import RC from './RC';
import ASTHistory from './ASTHistory';
import Buttons from './Buttons'
import { useSettings } from './SettingsProvider'
import SettingsMenu from './SettingsMenu'
import {ParseOptions} from "sfl_wasm_lib";

function App() {
  const { isLightTheme, typecheckerEnabled, preludeEnable } = useSettings();
  const [rcs, setRcs] = useState<JSX.Element[]>([]);
  const [editorValue, setEditorValue] = useState("");
  const [errorString, setErrorString] = useState("");
  const [astHistory, setAstHistory] = useState<wasm.RawASTInfo[]>([]);
  const [selectedRcFromStringHistory, setSelectedRcFromStringHistory] = useState<string[]>([]);
  const [selectedRcToStringHistory, setSelectedRcToStringHistory] = useState<string[]>([]);
  const [settingsIsVisible, setSettingsIsVisible] = useState(false);
  const [multiple, setMultiple] = useState(false);
  const [lhsWidth, setLhsWidth] = useState(window.innerWidth / 2);

  const generateRCs = (ast: wasm.RawASTInfo, _multiple: boolean) => {
    try {
      const rcs = _multiple ? wasm.get_all_redexes(ast) : wasm.get_one_redex(ast);

      if (wasm.get_rcs_len(rcs) === 0) {
        setRcs([]);
        return;
      }

      const rc_callback = (rc_index: number) => {
        const from_string = wasm.get_rcs_from(rcs, rc_index);
        const to_string = wasm.get_rcs_to(rcs, rc_index);

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
        generateRCs(ast, _multiple);
      };

      const rc_elems = [];
      for (let i = 0; i < wasm.get_rcs_len(rcs); i++) {
        const from_string = wasm.get_rcs_from(rcs, i);
        const to_string = wasm.get_rcs_to(rcs, i);
        const message = wasm.get_rcs_msg1(rcs, i);
        rc_elems.push(<RC text={_multiple} key={i + 1} i={i} onClick={rc_callback} from={from_string} to={to_string} msg={message} />);
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

  const handleRun = (programInput: string, _multiple: boolean) => {
    setMultiple(_multiple);
    try {
      const ast = wasm.parse(programInput, new ParseOptions(typecheckerEnabled, preludeEnable));
      setAstHistory([ast]);
      generateRCs(ast, _multiple);
      setSelectedRcFromStringHistory([])
      setSelectedRcToStringHistory([])
      setErrorString("")
    } catch (e) {
      setErrorString(e as string)
      setRcs([])
      setAstHistory([])
      setSelectedRcFromStringHistory([])
      setSelectedRcToStringHistory([])
    }
  };

  const resetTo = (n: number) => {
    setAstHistory((prevAstHistory) => {
      const new_slice = prevAstHistory.slice(0, n);
      generateRCs(new_slice[new_slice.length - 1], multiple);
      return new_slice
    });
  }

  const separatorDragStart = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
  };

  const separatorDrag = (e: React.MouseEvent<HTMLDivElement>) => {
    const newWidth = e.clientX; // Get the mouse position relative to the viewport
    setLhsWidth(newWidth);
    console.log(newWidth)
    const container = document.getElementById('themeContainer');
    container?.style.setProperty("grid-template-columns", `${newWidth} 7px 7px 1fr`);
  };

  const themeContainerStyle : React.CSSProperties = {
    display: 'grid',
    gridTemplateRows: '1fr',
    gridTemplateColumns: `${lhsWidth}px 7px 7px 1fr`,
  }

  return (
    <div id="themeContainer" className={isLightTheme ? "dark" : 'light'} style={themeContainerStyle}>
      <SettingsMenu settingsIsVisible={settingsIsVisible} dismissSettings={() => setSettingsIsVisible(false)} />
      <div id="lhs">
        <div id="Title">
          <div id="TitleFlex">
            <h1>SFL Explorer</h1>
            <p> by </p>
            <a href='https://github.com/kiran-isaac' target='blank'>Kiran Sturt</a>
          </div>
        </div>
        <div id="inputContainer" draggable="true">
          <Input
            editorValue={editorValue}
            setEditorValue={setEditorValue}
          />
          <Buttons
            handleRun={handleRun}
            setEditorValue={setEditorValue}
            editorValue={editorValue}
            setSettingsIsVisible={setSettingsIsVisible}
            settingsIsVisible={settingsIsVisible}
          />
        </div>
      </div>

      <div id="separator1" onDragStart={separatorDragStart} onDrag={separatorDrag}></div>
      <div id="separator2" onDragStart={separatorDragStart} onDrag={separatorDrag}></div>

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
          <ASTHistory astHistory={astHistory} resetTo={resetTo} rcToHistory={selectedRcToStringHistory} rcFromHistory={selectedRcFromStringHistory} />
        </div>
      </div>
    </div>
  );
}

export default App
