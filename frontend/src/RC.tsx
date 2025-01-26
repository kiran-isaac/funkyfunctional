interface RCProp {
    i: number;
    onClick: (rc_index: number) => void;
    from: string;
    to: string;
}

const RC: React.FC<RCProp> = ({ i, onClick, from, to }) => {
    return <><button
        className="rc"
        onClick={() => onClick(i)}
    >
        <pre className="from">{from}</pre> <pre id="rarrow">&rArr;</pre> <pre className="to">{to}</pre>
    </button><br /></>;
}

export default RC   