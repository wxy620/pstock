import React from "react";

export interface StockHistData{
    symbol: string;
    name: string;
    date: string;
    market?: string;
    /**
     * 最新价格
     */
    new_price?: number;

    open: number;
    close: number;

    high: number;
    low: number;

    /**
     * 涨跌幅
     */
    pct_chg: number;
    /**
     * 涨跌额
     */
    price_change: number;
    /**
     * 成交量
     */
    volume: number;
    /**
     * 成交额
     */
    turnover: number;

    /**
     * 震幅
     */
    amplitude: number;
    /**
     * 换手率
     */
    turnover_rate: number;

    kp_indicators?: string;
}
//{"cDL2Crows":0,"cDL3BlackCrows":0,"cDLDarkCloudCover":0,"cDLDojiStar":0,"cDLEveningDojiStar":0,"cDLIdentical3Crows":0,"cDLUpSideGap2Crows":0}
export type KpIndicators = {
    cDL3BlackCrows?: number,
}


export interface StockHistDataEx extends  StockHistData{
    key: number;
}


export interface StockHistMinData{
    symbol?: string;
    date?: string;
    timestamp: number;
    open: number;
    close: number;
    high: number;
    low: number;
    volume: number;
    turnover: number;
    newPrice?: number;
}


export interface StockBasic{
    symbol: string;
    name: string;
    market: string;
    publish_date: string;
}