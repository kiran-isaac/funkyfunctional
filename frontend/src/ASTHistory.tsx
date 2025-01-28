import * as wasm from 'sfl_wasm_lib'
import * as diff from 'diff';

interface ASTHistoryProps {
    astHistory: wasm.RawASTInfo[]
}

const ASTHistory = ({ astHistory }: ASTHistoryProps) => {
    const astStrings = [];
    for (let i = 0; i < astHistory.length; i++) {
        astStrings.push(wasm.main_to_string(astHistory[i]) + "\n");
    }

    if (astStrings.length === 0) {
        return <></>;
    }

    // Get diffs between each string
    // Get diffs between each string
    const astLIs = [];
    for (let i = astStrings.length - 2; i >= 0; i--) {
        const list : JSX.Element[] = [];
        const previous = astStrings[i + 1];
        const current = astStrings[i];
        diff.diffWords(current, previous).forEach((part, index) => {
            const color = part.added ? 'green' : 'white';
            if (part.removed) {
                return;
            }
            list.push(
                <span key={index} style={{ color }}>
                    {part.value}
                </span>
            );
        });
        astLIs.push(<li key={i + 1}>{list}<br/></li>);
    }
    astLIs.push(<li key={0}>{astStrings[0]}</li>);
    // astLIs = astLIs.reverse();   

    return (
        <ul id="ASTHistory">
            {astLIs}
        </ul>
    );
}

export default ASTHistory