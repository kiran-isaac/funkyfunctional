import { useState } from 'react';
import definitionString from './../../definition.txt?raw';
import './help.css';

function definition() {
    const [visible, setVisible] = useState(true);
    return <div id="definition" style={{ display: visible ? "block" : "none" }}>
        <button id="definitiondissmis" onClick={() => {
            setVisible(!visible);
        }}>X</button>
        <pre>{definitionString}</pre>
    </div>;
}

export default definition;