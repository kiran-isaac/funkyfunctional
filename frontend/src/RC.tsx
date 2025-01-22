interface RCProp {
    i: number;
    onClick: (rc_index: number) => void;
    from: string;
    to: string;
}

const RC: React.FC<RCProp> = ({ i, onClick, from, to }) => {
    return <><button onClick={() => onClick(i)}>
        {i}{") "}{from} {"->"} {to}
    </button><br /></>;
}

export default RC   