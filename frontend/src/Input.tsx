import "./input.css";
import { Controlled as CodeMirror } from 'react-codemirror2';
import { SetStateAction, useCallback, useEffect, useState } from "react";
import "codemirror/lib/codemirror.css";
import "codemirror/theme/monokai.css";
import * as wasm from "sfl_wasm_lib";

const eg_programs = import.meta.glob("./../../examples/*", {
    query: '?raw',
    import: 'default',
});


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


function ProgramDropdown({ inbuiltPrograms, setEditorValue }: { inbuiltPrograms: Map<string, string>, setEditorValue: (x: string) => void }) {
    const name_options: JSX.Element[] = [];
    inbuiltPrograms.forEach((_, name) => {
        name_options.push(<option key={name} value={name}>{name}</option>);
    });
    const onChange = () => {
        const e = document.getElementById("program_dropdown") as HTMLSelectElement;
        const val = e?.value;
        if (val == "__local__") {
            setEditorValue(localStorage.getItem("program") || "");
        }
        setEditorValue(inbuiltPrograms.get(val) || "")
    };

    return <select onChange={onChange} id="program_dropdown">
        <optgroup label="Inbuilt">{name_options}</optgroup>
        <option value="__local__">Local</option>
    </select>
}

interface InputProps {
    onRunMultiple: (editorValue: string) => void;
    onRunSingle: (editorValue: string) => void;
}

function Input({ onRunMultiple, onRunSingle }: InputProps) {
    const [editorValue, setEditorValue] = useState("");
    const [egProgramsMap, setEgProgramsMap] = useState<Map<string, string>>(new Map());

    useEffect(() => {
        const loadPrograms = async () => {
            const eg_programs_map: Map<string, string> = new Map();
            for (const path in eg_programs) {
                const program = await eg_programs[path]();

                const eg_name = path.split('\\').pop()?.split('/').pop()?.replace(".sfl", "");
                if (eg_name == undefined) { return; }
                eg_programs_map.set(eg_name, program as string)
            }

            setEgProgramsMap(eg_programs_map)
        };
        loadPrograms();
    }, []);

    // eslint-disable-next-line
    const editorOnChange = useCallback((val: SetStateAction<string>, _: any) => {
        // override local storage 
        localStorage.setItem("program", val.toString());
        const e = document.getElementById("program_dropdown") as HTMLSelectElement;
        e.value = "__local__"
        setEditorValue(val);
    }, []);

    return (
        <>
            <div id="ProgramInput">
                <PreludeDropdown />
                    <CodeMirror
                        value={editorValue}
                        className="code-mirror-wrapper"
                        options={
                            {
                                mode: 'scheme',
                                theme: 'monokai',
                                lineNumbers: true,
                                tabSize: 2,
                                lineWrapping: true,
                                
                                matchBrackets: true,
                                autoCloseBrackets: true,
                            }
                        }
                        onBeforeChange={(_, data, value) => {
                            editorOnChange(value, data);
                        }}
                    />
            </div>
            <div id="Buttons">
                <ProgramDropdown inbuiltPrograms={egProgramsMap} setEditorValue={setEditorValue} />
                <button className="runbutton" id="RunButtonSingle" onClick={() => onRunSingle(editorValue)}>Lazy</button>
                <button className="runbutton" id="RunButtonMultiple" onClick={() => onRunMultiple(editorValue)}>Free Choice</button>
            </div>
        </>
    );
}

export default Input