import "./lhs.css";
import { Controlled as CodeMirrorControlled } from 'react-codemirror2';
import {useEffect, useState} from "react";
import "codemirror/lib/codemirror.css";
import "./editor_themes/dark.css";
import "./editor_themes/light.css";
import "./sfl_codemirror.js";
import * as wasm from "sfl_wasm_lib";
import { useSettings } from "./SettingsProvider.js";

function PreludeDropdown() {
    const { isLightTheme } =  useSettings();

    const [isVisible, setIsVisible] = useState(false);
    const [preludeValue, setPreludeValue] = useState(wasm.get_prelude());

    useEffect(() => {
        const button = document.getElementById("prelude_dropdown_button");
        const prelude = document.getElementById("prelude");
        const editor = document.querySelector(".code-mirror-wrapper") as HTMLElement;
        const preludeEditor = document.querySelector(".prelude_code-mirror-wrapper") as HTMLElement;

        if (button == null || prelude == null) { return; }

        const handleClick = () => {
            if (isVisible) {
                prelude.style.height = "0";
                editor.style.height = "calc(100% - 49px)";
                setIsVisible(false);
                setPreludeValue(preludeValue);
            } else {
                prelude.style.height = "45vh";
                editor.style.height = "calc(100% - 49px - 45vh)";
                setIsVisible(true);
                preludeEditor.style.display = "block";
            }
        };

        button.addEventListener("click", handleClick);

        return () => {
            button.removeEventListener("click", handleClick);
        };
    }, [isVisible]);

    return (
        <div id="prelude_dropdown">
            <button type="button" id="prelude_dropdown_button">See Prelude</button>
            <div id="prelude">
                <CodeMirrorControlled
                    value={preludeValue}
                    className="prelude-code-mirror-wrapper"
                    options={
                        {
                            mode: "sfl",
                            theme: isLightTheme ? "sfl_dark" : "sfl_light",
                            lineNumbers: true,
                            tabSize: 2,
                            lineWrapping: true,
                            readOnly: true,
                            indentWithTabs: false,
                            viewportMargin: Infinity,
                        }
                    }
                    onBeforeChange={() => {}}
                />
            </div>
        </div>
    );
}

interface InputProps {
    editorValue: string;
    setEditorValue: (x: string) => void;
}

function Input({ editorValue, setEditorValue }: InputProps) {
    const { isLightTheme, preludeEnable } = useSettings();
    return (
        <>
            <div id="ProgramInput">
                <PreludeDropdown />
                <CodeMirrorControlled
                    value={editorValue}
                    className="code-mirror-wrapper"
                    options={
                        {
                            mode: preludeEnable ? "sfl" : "sfl_no_prelude",
                            theme: isLightTheme ? "sfl_light" : "sfl_dark",
                            lineNumbers: true,
                            tabSize: 2,
                            lineWrapping: true,
                        }
                    }
                    onBeforeChange={(_0, _1, value) => {
                        localStorage.setItem("program", value.toString());
                        const e = document.getElementById("program_dropdown") as HTMLSelectElement;
                        e.value = "__local__"
                        setEditorValue(value);
                    }}
                />
            </div>
        </>
    );
}

export default Input