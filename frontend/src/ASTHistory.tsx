import * as wasm from 'sfl_wasm_lib'

interface ASTHistoryProps {
    astHistory: wasm.RawASTInfo[];
    rcFromHistory: string[];
    rcToHistory: string[];
}

const ASTHistory = ({ astHistory, rcFromHistory, rcToHistory }: ASTHistoryProps) => {
    if (astHistory.length == 0) {
        return <></>;
    }

    const astLIs = [];

    for (let i = astHistory.length - 1; i >= 1; i--) {
        const prev = astHistory[i - 1];
        const current = astHistory[i];

        const diff = wasm.diff(prev, current);

        const spanList = [];

        for (let j = 0; j < wasm.get_diff_len(diff); j += 1) {
            if (wasm.diff_is_similar(diff, j)) {
                spanList.push(<span>{wasm.diff_get_similar(diff, j)}</span>);
            } else {
                const pair = wasm.diff_get_diff(diff, j);
                const str1 = wasm.stringpair_one(pair);
                const str2 = wasm.stringpair_two(pair);
                spanList.push(<span className="old">{str1}</span>);
                spanList.push(<span className="new">{str2}</span>);
            }
        }

        astLIs.push(<li className='expr_history' key={i}>{spanList}<br/></li>)
    }   

    astLIs.push(<li className='expr_history' key={0}>{wasm.main_to_string(astHistory[0])}</li>);

    return (
        <ul id="ASTHistory">
            {astLIs}
        </ul>
    );
}

export default ASTHistory
