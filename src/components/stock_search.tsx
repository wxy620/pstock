import React, {useEffect, useState} from "react";
import {AutoComplete, AutoCompleteProps, Button, Flex, Input, Space} from "antd";
import {SearchOutlined, StarOutlined} from "@ant-design/icons";
import {invoke} from "@tauri-apps/api/core";
import {StockBasic, StockHistDataEx} from "../model/stock.ts";
import {groupBy, marketStr} from "../util/utils.ts";
import {SorterResult} from "antd/es/table/interface";
import {observer} from "mobx-react-lite";
import {makeAutoObservable} from "mobx";



export class StockSearchStore {
    sortOrders?: Sorter
    searchParams?: SearchParams
    result?: SearchResult
    constructor() {
        makeAutoObservable(this)
    }

    setSortOrders(value: Sorter){
        this.sortOrders = value
    }

    setSearchParams(value: SearchParams) {
        this.searchParams = value
    }

    setResult(value: SearchResult){
        this.result = value
    }
}

export  const searchStore = new StockSearchStore()

const Title: React.FC<Readonly<{ title?: string }>> = (props) => (
    <Flex align="center" justify="space-between">
        {props.title}
        <a href="https://www.google.com/search?q=antd" target="_blank" rel="noopener noreferrer">
            more
        </a>
    </Flex>
);


type SortOrder = {
    field?: string;
    order?: 'descend' | 'ascend' | null;
}
type SearchParams ={
    market?: 'sh' | 'sz' | null;
    followOnly: boolean;
    symbol?: string,
}

export  type SearchResult = {
    inputText?: string;
    symbol?: string;
    data?: StockHistDataEx[]
}

export type Sorter = SorterResult<any>[] | SortOrder[] | undefined

const StockSearch =  observer(({ searchStore }: {searchStore: StockSearchStore}) =>  {

    const [searchText, setSearchText] = useState<string>()

    const [options, setOptions] = useState<AutoCompleteProps['options']>([]);


    const fetchData = () => {

        // let transformAToB = (input: SorterResult<any>): SortOrder => {
        //     return {
        //         field: input.field as string,
        //         order: input.order,
        //     }
        // }
        const sortFields = searchStore.sortOrders
        let sortOrders =
            sortFields?.map((x)=> x as unknown as SortOrder);
        console.log("sortOrders:", sortOrders);
        //console.log(`请求后台股票列表，searchParams:`, searchParams)
        //console.log("sortOrders:", sortOrders)

        (async()=>{
            await invoke<StockHistDataEx[]>('query_stock_list', {
                sortOrders: sortOrders ?? null,
                searchParams: searchStore.searchParams,
            }).then((message) =>{
                searchStore.setResult({
                    inputText: searchText,
                    symbol: searchStore.searchParams?.symbol,
                    data: message
                })
            }).catch((_)=>{})
        })();
    };

    useEffect(fetchData, [
        searchStore.searchParams, searchStore.sortOrders
    ]);


    const handleSearch = async (value: string) => {
        setSearchText(value)
        //setOptions(value ? searchResult(value) : []);
        await invoke<StockBasic[]>('fuzzy_query', {q: value}).then((message) => {
            //console.log(`${message}`)
            const result= groupBy(message, i => i.market);
            //console.log(`${result}`)
            let tempOptions = []
            for (let k in result) {
                const option = buildSelectOption(k, result[k])
                tempOptions.push(option);
            }
            setOptions(tempOptions)

        }).catch((_)=>{})
    };

    const buildSelectOption = (k: string, list: StockBasic[]) =>{
        const options = list.map((v, _) =>{
            return renderItem(v.name, v.symbol);
        });
        return {
            label: <Title title={
                marketStr(k)
            } />,
            options
        };
    };

    const renderItem = (name: string, symbol:  string| number) => ({
        value: symbol,
        label: (
            <Flex align="center" justify="space-between">
                {symbol}
                <Flex gap="small">
                    {name}
                    <StarOutlined />
                </Flex>
            </Flex>
        ),
    });

    const handleClick= (_: React.MouseEvent<HTMLButtonElement>) =>{
        const market = searchStore.searchParams?.market
        const followOnly = searchStore.searchParams?.followOnly?? false
        searchStore.setSearchParams ({
            symbol: searchText,
            market: market,
            followOnly: followOnly ?? false
        });
    };

    return (
        <>
            <Space.Compact  size={"large"}>
                <AutoComplete
                    popupClassName="certain-category-search-dropdown"
                    popupMatchSelectWidth={500}
                    options={options}
                    style={{width:250}}
                    onSearch={handleSearch}
                    onSelect={(v:string) =>{setSearchText(v)}}
                >
                    <Input style={{height:40}} placeholder="请输入例如: `600000` ｜ `上证`"  />
                </AutoComplete>
                <Button icon={<SearchOutlined />}  target="_blank"
                        onClick={handleClick} />
            </Space.Compact>
        </>
    );

})

export default  StockSearch;