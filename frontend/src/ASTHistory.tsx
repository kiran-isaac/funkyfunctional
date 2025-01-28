import * as wasm from 'sfl_wasm_lib'

interface ASTHistoryProps {
    astHistory: wasm.RawASTInfo[]
}

const ASTHistory = ({ astHistory }: ASTHistoryProps) => {
    const astStrings = [];
    for (let i = 0; i < astHistory.length; i++) {
        astStrings.push(wasm.to_string(astHistory[i]) + "\n");
    }
    return (
        <ul id="ASTHistory">
            {astStrings.map((astString, i) => <li key={i}>{astString}<br /></li>)}
        </ul>
    );
}

export default ASTHistory