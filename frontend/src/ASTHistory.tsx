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
            <div><pre>{to}</pre></div>
        </div>
    );
}

interface ASTHistoryProps {
    astHistory: wasm.RawASTInfo[];
    resetTo: (n: number) => void;
    rcFromHistory: string[];
    rcToHistory: string[];
}

const ASTHistory = ({ astHistory, resetTo, rcFromHistory, rcToHistory }: ASTHistoryProps) => {
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

        for (let j = 0; j < wasm.get_diff_len(diff); j += 1) {
            if (wasm.diff_is_similar(diff, j)) {
                exprSpanList.push(<span>{wasm.diff_get_similar(diff, j)}</span>);
            } else {
                const pair = wasm.diff_get_diff(diff, j);
                const str2 = wasm.stringpair_two(pair);
                exprSpanList.push(<span className="changed">{str2}</span>);
            }
        }

        diffSpanList.push(<div><DiffDisplay from={rcFromHistory[i-1]} to={rcToHistory[i-1]}></DiffDisplay></div>);
        astLIs.push(<li className='expr_history' key={i}><div className="exprListing"><pre>{exprSpanList}</pre></div><pre>{diffSpanList}</pre></li>)
    }

    astLIs.push(<li className='expr_history' key={0}><pre>{wasm.main_to_string(astHistory[0])}</pre></li>);

    const listener = (e: KeyboardEvent) => {
        const quickProgress = () =>         {
            const button = document.getElementById("first_button");
            if (button == null) {
                return
            }
            button.click();
        };
        switch (e.key) {
        case "ArrowLeft":
            if (astHistory.length > 1) {
                resetTo(astHistory.length - 1);
            }
            break;
        case "ArrowRight":
            quickProgress();
            break;
        }
    };

    document.addEventListener("keydown", listener, {once: true});

    return (
        <div id="ASTHistoryWrapper">
            <table id="ASTHistory">
                <tbody>
                    {astLIs.map((li, index) => (
                        <tr key={astLIs.length - index - 1} className={index == 0 ? 'top' : ''}>
                            <td className='index'><p>{astLIs.length - index - 1}</p></td>
                            <td className='ast' onClick={() => resetTo(astLIs.length - index)}>{li}</td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}

export default ASTHistory
