import "./lhs.css";
import { Controlled as CodeMirrorControllerd } from 'react-codemirror2';
import { useEffect, useState } from "react";
import "codemirror/lib/codemirror.css";
import "codemirror/theme/monokai.css";
import "codemirror/addon/mode/simple.js";
import CodeMirror from "codemirror";
import * as wasm from "sfl_wasm_lib";

CodeMirror.defineSimpleMode("simplemode", {
    // The start state contains the rules that are initially used
    start: [
        // The regex matches the token, the token property contains the type
        { regex: /"(?:[^\\]|\\.)*?(?:"|$)/, token: "string" },
        // You can match multiple tokens at once. Note that the captured
        // groups must span the whole string in this case
        {
            regex: /(function)(\s+)([a-z$][\w$]*)/,
            token: ["keyword", null, "variable-2"]
        },
        // Rules are matched in the order in which they appear, so there is
        // no ambiguity between this one and the one above
        {
            regex: /(?:function|var|return|if|for|while|else|do|this)\b/,
            token: "keyword"
        },
        { regex: /true|false|null|undefined/, token: "atom" },
        {
            regex: /0x[a-f\d]+|[-+]?(?:\.\d+|\d+\.?\d*)(?:e[-+]?\d+)?/i,
            token: "number"
        },
        { regex: /\/\/.*/, token: "comment" },
        { regex: /\/(?:[^\\]|\\.)*?\//, token: "variable-3" },
        // A next property will cause the mode to move to a different state
        { regex: /\/\*/, token: "comment", next: "comment" },
        { regex: /[-+\/*=<>!]+/, token: "operator" },
        // indent and dedent properties guide autoindentation
        { regex: /[\{\[\(]/, indent: true },
        { regex: /[\}\]\)]/, dedent: true },
        { regex: /[a-z$][\w$]*/, token: "variable" },
        // You can embed other modes with the mode property. This rule
        // causes all code between << and >> to be highlighted with the XML
        // mode.
        { regex: /<</, token: "meta", mode: { spec: "xml", end: />>/ } }
    ],
    // The multi-line comment state.
    comment: [
        { regex: /.*?\*\//, token: "comment", next: "start" },
        { regex: /.*/, token: "comment" }
    ],
    // The meta property contains global information about the mode. It
    // can contain properties like lineComment, which are supported by
    // all modes, and also directives like dontIndentStates, which are
    // specific to simple modes.
    meta: {
        dontIndentStates: ["comment"],
        lineComment: "//"
    }
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
                                mode: "simplemode",
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