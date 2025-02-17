import {StockHistData, StockHistMinData} from "./stock.ts";

export type IPayload =

    | {
    event: 'dashboard';
    data: StockHistData;
    msgId? : string;
}
    | {
    event: 'kLineRT';
    data: StockHistMinData[];
    msgId? : string;
    timestamp?: number;
    isFull?: number;
}

    | {
    event: 'started';
    data: {
        url: string;
        downloadId: number;
        contentLength: number;
    };
}
    | {
    event: 'progress';
    data: {
        downloadId: number;
        chunkLength: number;
    };
}
    | {
    event: 'finished';
    data: {
        downloadId: number;
    };
};



