import React, {useEffect, useRef, useState} from 'react';
// 引入样式
import {CandleType, Chart, dispose, init, KLineData, TooltipShowRule, TooltipShowType} from "klinecharts";
import {Radio, RadioChangeEvent, Space} from "antd";
import './klinechart.css'
import generatedMockDataList from "./generatedMockDataList.ts";
import {invoke} from "@tauri-apps/api/core";
import {KpIndicators, StockHistData} from "../model/stock.ts";
import {listen} from "@tauri-apps/api/event";
import {IPayload} from "../model/payload.ts";
import {searchStore} from "./stock_search.tsx";
import {observer} from "mobx-react-lite";

function getTooltipOptions (candleShowType: TooltipShowType, candleShowRule: TooltipShowRule,
                            indicatorShowType: TooltipShowType, indicatorShowRule: TooltipShowRule) {
    return {
        candle: {
            type: 'candle_solid' as CandleType,
            tooltip: {
                showType: candleShowType,
                showRule: candleShowRule,
                offsetTop: 2
                // custom: (data: CandleTooltipCustomCallbackData) => {
                //     const { prev, current } = data
                //     const prevClose = (prev?.close ?? current.open)
                //     const change = (current.close - prevClose) / prevClose * 100
                //     return [
                //         { title: 'open', value: current.open.toFixed(2) },
                //         { title: 'close', value: current.close.toFixed(2) },
                //         {
                //             title: 'Change: ',
                //             value: {
                //                 text: `${change.toFixed(2)}%`,
                //                 color: change < 0 ? '#EF5350' : '#26A69A'
                //             }
                //         }
                //     ]
                // }
            }
        },
        indicator: {
            tooltip: {
                showRule: indicatorShowRule,
                showType: indicatorShowType,
                offsetTop:0,
            }
        }
    }
}


type IStock = {
    name?: string,
    market?: string ,
    symbol?: string
}

const KLineChart: React.FC<{ menuCollapsed: boolean }> = ({menuCollapsed}) => {

    const subIndicators = ['VOL', 'MACD', 'KDJ']

    const [tabPosition, setTabPosition] = useState<string>('RT');

    const chartRef = useRef<Chart | null>()
    const paneId = useRef<string>('')

    const divRef = useRef<HTMLDivElement | null>(null)

    // const [kLineData, setKLineData] = useState<KLineData[]>([])

    //方形: rect  , 标准(在头部): standard
    //open close vol 等
    const [candleShowType, setCandleShowType] = useState('standard')
    // 跟随指标: follow_cross //总是显示: always  不显示 none
    const [candleShowRule, setCandleShowRule] = useState('always')
    const [indicatorShowType, setIndicatorShowType] = useState('rect')
    const [indicatorShowRule, setIndicatorShowRule] = useState('always')

    const [selectedData, setSelectedData] = useState<IStock>({
    })


    const handleTabChange = (e:RadioChangeEvent) =>{
        setTabPosition(e.target.value)
    }

    //symbol: '000004', name: '国华网安', market:'sz'
    async function loadData(){
        let dataList: KLineData[]
        switch (tabPosition){
            case 'daily': {
                let _tmpDataList: KLineData[] = []
                await invoke('query_kline_hist', {...selectedData})
                    .then((list) => {
                        const array = list as StockHistData[]
                        for (const hisData of array) {
                            const kLineData: KLineData = {
                                open: hisData.open,
                                low: hisData.low,
                                high: hisData.high,
                                close: hisData.close,
                                volume: hisData.volume,
                                turnover: hisData.turnover_rate,
                                timestamp:  Date.parse(hisData.date +"T15:00:00"),
                            }

                            if(hisData.kp_indicators != undefined){
                                kLineData.indicators = JSON.parse(hisData.kp_indicators)
                                console.log(kLineData)
                            }

                            _tmpDataList.push(kLineData)
                        }
                    })
                    .catch((error) => console.error(error))

                dataList = _tmpDataList
                console.log("load daily kline data, size:", dataList.length)
                break;
            }
            default:
                dataList = generatedMockDataList()
                break;
        }

        return dataList
    }

    // 监听窗口大小变化
    useEffect(() => {
        const resizeObserver = new ResizeObserver(entries => {
            for (let entry of entries) {
                const { width, height } = entry.contentRect;
                console.log(`${ entry.target.className} ,Size changed. New width: ${width}, New height: ${height}`);
                if(entry.target.className === 'k-line-chart-container'){
                    chartRef.current?.resize()
                }
            }
        });
        if (divRef.current) {
            resizeObserver.observe(divRef.current);
        }
        return () => {
            if (divRef.current) {
                resizeObserver.unobserve(divRef.current);
            }
        };
    }, []);



    //默认初始化事件，参数为空只执行1次，
    useEffect(() => {
        chartRef.current = init('stock-kline-chart')
        chartRef.current?.setTimezone("Asia/Shanghai")
        chartRef.current?.setLocale('zh-CN')
        chartRef.current?.createIndicator('MA', false, { id: 'candle_pane' })
        paneId.current = chartRef.current?.createIndicator('VOL', false, { height: 80 }) as string

        const handleWindowResize = () => {
            chartRef.current?.resize()
        };
        window.addEventListener('resize', handleWindowResize);
        const unListen = (async()=>{
            return await listen<IPayload>('sync_kline_event', (message) => {
                let payload = message.payload;
                console.log(`${payload.event}`);
                if(payload.event == 'kLineRT'){
                    const list = payload.data;
                    let klines = list.map((item,index) =>{
                        const kLineData: KLineData = {
                            open: item.open,
                            low: item.low,
                            high: item.high,
                            close: item.close,
                            volume: item.volume,
                            timestamp:  item.timestamp,
                        };
                        return kLineData;
                    })
                    const isFull = payload.isFull ?? 0;
                    if(klines.length > 1  && isFull ){
                        console.log("resize now!", klines.length)
                        chartRef.current?.applyNewData(klines)
                    }else{
                        chartRef.current?.updateData(klines[0])
                    }
                }
            });
        })();

        return () => {
            dispose('stock-kline-chart')
            window.removeEventListener('resize', handleWindowResize)
            unListen.then((f)=>{
                console.log("destroy listen ...");
                f();
            })
            invoke('un_sync_kline_data').then(r => {})
        }
    }, []);

    //监听样式发生变化
    useEffect(() => {
        chartRef.current?.setStyles(getTooltipOptions(
            candleShowType as TooltipShowType, candleShowRule as TooltipShowRule,
            indicatorShowType as TooltipShowType, indicatorShowRule as TooltipShowRule
        ))
    }, [candleShowType, candleShowRule, indicatorShowType, indicatorShowRule])


    //监听tab栏发生变化
    useEffect(() => {
        //自执行函数
        (async () => {
            chartRef.current?.clearData()
            if(tabPosition === 'RT'){
                await invoke('sync_kline_data', {symbol: selectedData?.symbol}).then(()=>{})
            }else{
                //TODO await invoke un_sync_kline_data method
                console.log({...selectedData})
                let data = await loadData();
                chartRef.current?.applyNewData(data)
            }
            //TOO change_event_data
        })();
    }, [tabPosition])

    // //K线数据发生变化
    // useEffect(()=>{
    //     chartRef.current?.applyNewData(kLineData)
    // },[kLineData])


    useEffect(() => {
        //自执行函数
        (async () => {
            if(tabPosition === 'RT'){
                if(selectedData.symbol == undefined){
                    chartRef.current?.clearData()
                    return;
                }
                await invoke('sync_kline_data', {symbol: selectedData?.symbol}).then(()=>{})
            }else{
                //TODO await invoke un_sync_kline_data method
                //console.log({...selectedData})
                let data = await loadData();
                chartRef.current?.applyNewData(data)
            }
            //TOO change_event_data
        })();
    }, [selectedData]);

    // demo 一个父子组件交互的例子
    useEffect(() => {
        console.log("menuCollapsed:", menuCollapsed)
    }, [menuCollapsed]);


    useEffect(() => {
        const result = searchStore.result
        console.log(result?.inputText)
        if(result === undefined || result?.inputText === undefined || result?.inputText === ""){
            setSelectedData({})
            return
        }

        const d = result.data;
        if (d != undefined && d.length > 0){
            setSelectedData({
                symbol: d[0].symbol,
                name: d[0].name,
                market: d[0].market,
            })
            return
        }

        setSelectedData({})

    }, [searchStore.result]);

    return(
        <>
            <div  ref={divRef} className="k-line-chart-container" style={{width:'100%'}}>
                <div className="k-line-chart-tab">
                    <Space>
                        <Radio.Group value={tabPosition} onChange={handleTabChange}>
                            <Radio.Button value="RT">分时</Radio.Button>
                            <Radio.Button value="daily">日K</Radio.Button>
                            <Radio.Button value="weekly">周K</Radio.Button>
                            <Radio.Button value="monthly">月K</Radio.Button>
                        </Radio.Group>
                    </Space>
                </div>
                <div id="stock-kline-chart"  className="k-line-chart "  />
                <div className="k-line-chart-menu-container" >
                    <span style={{paddingRight: 10, paddingLeft: 12}}>副图指标</span>
                    {
                        subIndicators.map(type => {
                            return (
                                <button
                                    key={type}
                                    onClick={_ => {
                                        chartRef.current?.createIndicator(type, false, {id: paneId.current})
                                    }}>
                                    {type}
                                </button>
                            )
                        })
                    }
                    <button
                        onClick={() => {
                            const dataList = chartRef.current?.getDataList() ?? []
                            for(let x of dataList){
                                const kData = x as KLineData
                                const val:KpIndicators = kData.indicators as KpIndicators
                                if(val != undefined && val.cDL3BlackCrows != undefined){
                                    if(val.cDL3BlackCrows < 0){
                                        chartRef.current?.createOverlay({
                                            name: 'simpleAnnotation',
                                            extendData: '三只乌鸦',
                                            points: [{ timestamp: kData.timestamp, value: kData.high }]
                                        })
                                    }
                                }
                            }
                        }}>
                        内置注解
                    </button>
                </div>
            </div>
        </>
    )
}


export default observer(KLineChart)