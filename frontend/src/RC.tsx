interface RCProp {
    onClick: () => void;
    from: string;
    to: string;
    i: number;
}

const RC: React.FC<RCProp> = ({ onClick, from, to, i }) => {
    return <button onClick={onClick}>
        {i}{") "}{from} {"->"} {to}
    </button>
}

export default RC   