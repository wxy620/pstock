import React, {useEffect} from 'react';
import type {TableColumnsType} from 'antd';
import { Flex, Space, Statistic, Table, TableProps} from 'antd';
import {ArrowDownOutlined, ArrowUpOutlined,} from '@ant-design/icons'
import {volumeFormat} from "../util/utils.ts";
import {StockHistDataEx} from "../model/stock.ts";
import {SorterResult} from "antd/es/table/interface";
import {searchStore} from "./stock_search.tsx";
import {observer} from "mobx-react-lite";
import {Link} from "react-router-dom";
import {menuStore} from "./mysider.tsx";

const getNumStyle = (record:StockHistDataEx) => {
    const change_rate = record.pct_chg
    let _color = '#cf1322'
    let _prefix = <ArrowUpOutlined />;
    if (change_rate){
        if(change_rate < 0){
            _color = '#3f8600'
            _prefix = <ArrowDownOutlined />
        }
    }
    return {_color, _prefix}
}

const columns: TableColumnsType<StockHistDataEx> = [
    {
        title: '股票',
        width: 100,
        dataIndex: 'name',
        key: 'name',
        fixed: 'left',
        render: (_text, record)=> {
            //console.log(record.name , record.symbol)
            return (
                <Flex vertical>
                    <span>{record.name}</span>
                    <span style={{fontSize: 10, color:"#999"}} >{record.symbol}</span>
                </Flex>

            )
        }
    },
    {
        title: '最新',
        width: 50,
        dataIndex: 'new_price',
        key: 'new_price',
        sorter: {
            compare: (a, b) => (a.new_price ?? 0) - (b.new_price ?? 0),
            multiple: 1,
        },
        render:(_text, record) =>{
           const {_color, } = getNumStyle(record);
            return (
                <Statistic
                    value={record.new_price}
                    precision={2}
                    valueStyle={{ color:_color }}
                    // prefix={_prefix}
                />
            )
        }
    },
    {
        title: '涨跌幅',
        width: 50,
        dataIndex: 'pct_chg',
        key: 'pct_chg',
        sorter: {
            compare: (a, b) => a.pct_chg - b.pct_chg ,
            multiple: 2,
        },
        render:(_text, record) =>{
            const {_color, _prefix} = getNumStyle(record);
            return (
                <Statistic
            value={record.pct_chg}
            precision={2}
            valueStyle={{ color:_color }}
            // prefix={_prefix}
            suffix="%"
                />
            )
        }
    },
    {
        title: '成交量',
        width: 50,
        dataIndex: 'volume',
        key: 'volume',
        sorter: {
            compare: (a, b) => a.volume - b.volume ,
            multiple: 3,
        },
        render:(_text, record) =>{
            let volumeStr = String(record.volume)
            if(record.volume){
                volumeStr = volumeFormat(record.volume)
            }
            return (
                <Statistic
                    value={volumeStr}
                    // prefix={_prefix}

                />
            )
        }
    },

    {
        title: 'action',
        key: 'operation',
        fixed: 'right',
        width: 100,
        render: (value, record, index) =>
            <Link to="/kline" onClick={()=>{
                searchStore.setSearchParams({
                    symbol: record.symbol,
                    followOnly: false,
                })
                console.log("menu change click",menuStore.selectKey)
                menuStore.setKey(["2"])
            }}>K 线</Link>,
    },
];


const StockTable: React.FC = () => {
    const handleTblChange: TableProps<StockHistDataEx>['onChange'] = (pagination, filters, sorter, extra) => {
        let sortFields: SorterResult<StockHistDataEx>[] | undefined = [];
        if (Array.isArray(sorter)){
            sortFields = sorter;
        }else{
            if (sorter.field !== undefined){
                sortFields.push(sorter);
            }else{
                sortFields = undefined
            }
        }
        searchStore.setSortOrders(sortFields)
    };


    useEffect(() => {
        if (searchStore.result != undefined){
            const { symbol, inputText} = searchStore.result
            if(symbol != inputText){
                searchStore.setSearchParams({
                    symbol: inputText,
                    followOnly: false
                })
            }
        }
    }, []);
    return (
        <>
            <Space direction={"vertical"} style={{width:'100%'}}>
                {/*<Space.Compact  size={"large"}>*/}
                {/*    <StockSearch onMessage={handleMessage}/>*/}
                {/*</Space.Compact>*/}

                <Table<StockHistDataEx>
                    pagination={false}
                    columns={columns}
                    onChange={handleTblChange}
                    dataSource={searchStore.result?.data}
                    scroll={{ x: 'max-content' }}
                    style={{border:"1px solid #f0f0f0", borderRadius:8}}
                />
            </Space>
        </>
    );
};
export default  observer(StockTable);