import "./lhs.css";
import { Controlled as CodeMirrorControllerd } from 'react-codemirror2';
import { useEffect, useState } from "react";
import "codemirror/lib/codemirror.css";
import "codemirror/theme/monokai.css";
import "./sfl_codemirror";
import * as wasm from "sfl_wasm_lib";

function PreludeDropdown() {
    const prelude = wasm.get_prelude();

    const [isVisible, setIsVisible] = useState(false);

    useEffect(() => {
        const button = document.getElementById("prelude_dropdown_button");
        const prelude = document.getElementById("prelude");
        const editor = document.querySelector(".code-mirror-wrapper") as HTMLElement;

        if (button == null || prelude == null) { return; }

        const handleClick = () => {
            if (isVisible) {
                prelude.style.display = "none";
                prelude.style.height = "0";
                editor.style.height = "calc(100% - 45px)";
                setIsVisible(false);
            } else {
                prelude.style.display = "block";
                prelude.style.height = "45vh";
                editor.style.height = "calc(100% - 45px - 45vh)";
                setIsVisible(true);
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
            <div id="prelude" style={{ display: "none" }}>
                <h2>Prelude</h2><pre>{prelude}</pre>
            </div>
        </div>
    );
}

interface InputProps {
    editorValue: string;
    setEditorValue: (x: string) => void;
}

function Input({ editorValue, setEditorValue }: InputProps) {
    return (
        <>
            <div id="ProgramInput">
                <PreludeDropdown />
                    <CodeMirrorControllerd
                        value={editorValue}
                        className="code-mirror-wrapper"
                        options={
                            {
                                mode: "sfl",
                                theme: 'monokai',
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