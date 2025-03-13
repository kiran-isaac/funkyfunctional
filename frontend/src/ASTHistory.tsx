import * as wasm from 'sfl_wasm_lib'
import './ASTHistory.css'

interface DiffDisplayProps {
    from: string;
    to: string;
}

const DiffDisplay = ({ from, to }: DiffDisplayProps) => {
    return (
        <div className='center_area'>
            <div><pre>{from}</pre></div>
            <div id='divider'>{"▷*"}</div>
            {/* <div id='divider2'></div> */}
            <div><pre>{to}</pre></div>
        </div>
    );
}

interface ASTHistoryProps {
    astHistory: wasm.RawASTInfo[];
    resetTo: (n: number) => void;
}

const ASTHistory = ({ astHistory, resetTo }: ASTHistoryProps) => {
    if (astHistory.length == 0) {
        return <></>;
    }
    const astLIs = [];

    for (let i = astHistory.length - 1; i >= 1; i--) {
        const prev = astHistory[i - 1];
        const current = astHistory[i];

        const diff = wasm.diff(prev, current);

        const exprSpanList = [];
        const diffSpanList = [];
        const hasOccured : Set<string> = new Set();

        for (let j = 0; j < wasm.get_diff_len(diff); j += 1) {
            if (wasm.diff_is_similar(diff, j)) {
                exprSpanList.push(<span>{wasm.diff_get_similar(diff, j)}</span>);
            } else {
                const pair = wasm.diff_get_diff(diff, j);
                const str1 = wasm.stringpair_one(pair);
                const str2 = wasm.stringpair_two(pair);
                exprSpanList.push(<span className="changed">{str2}</span>);
                const setIdent = str1 + '\0' + str2;
                if (!hasOccured.has(setIdent)) {
                    diffSpanList.push(<div><DiffDisplay from={str1} to={str2}></DiffDisplay></div>);
                    hasOccured.add(setIdent);
                }
            }
        }

        astLIs.push(<li className='expr_history' key={i}><div className="exprListing"><pre>{exprSpanList}</pre></div><pre>{diffSpanList}</pre></li>)
    }

    astLIs.push(<li className='expr_history' key={0}><pre>{wasm.main_to_string(astHistory[0])}</pre></li>);

    return (
        <div id="ASTHistoryWrapper">
            <table id="ASTHistory">
                <tbody>
                    {astLIs.map((li, index) => (
                        <tr key={astLIs.length - index - 1} className={index == 0 ? 'top' : ''} onClick={() => resetTo(astLIs.length - index)}>
                            <td className='index'><p>{astLIs.length - index - 1}</p></td>
                            <td className='ast'>{li}</td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}

export default ASTHistory
