init('./wasm_weakauras_parser_bg.wasm').then(() => self.postMessage({
    message: 'initialized',
    data: null,
}));

self.onmessage = ({ data }) => {
    const parsed = parse(data);
    self.postMessage({
        message: 'completed',
        data: parsed,
    });
};
