import React, {useCallback, useEffect, useRef, useState} from 'react';
import {
    AppstoreOutlined,
    BarChartOutlined,
    CloudOutlined,
    ShopOutlined,
    TeamOutlined,
    UploadOutlined,
    UserOutlined,
    VideoCameraOutlined,
    MenuUnfoldOutlined,
    MenuFoldOutlined,
} from '@ant-design/icons';
import {Button, Flex, MenuProps} from 'antd';
import { Layout, Menu, theme } from 'antd';
import './App.css'
import {BrowserRouter, Routes, Route} from "react-router-dom";

import KLineChart from "./components/klinechart.tsx";
import RTKLineChart from "./components/rt-klinechart.tsx";
import StockTable from "./components/stocktable.tsx";

import {emit, listen} from "@tauri-apps/api/event";
import {IPayload} from "./model/payload.ts";
import Stock_search , {searchStore} from "./components/stock_search.tsx";
import MySider, {menuStore} from "./components/mysider.tsx";



const { Header, Content, Footer } = Layout;



const App: React.FC = () => {
    const [collapsed, setCollapsed] = useState(true);
    const {
        token: { colorBgContainer, borderRadiusLG },
    } = theme.useToken();

    useEffect(() => {

        // invoke('call_my_sidecar').then(r => {})

        emit('start_global', {msgId: new Date().getTime()}).then(()=>{})
        //   Listen emit
        const unListen = (async()=>{
            return await listen<IPayload>('global_event', (message) => {
                let payload = message.payload;
                console.log(
                    ` ${payload.event}`
                );
            });
        })();
        return () => {
            unListen.then((f)=>{
                console.log('unregister listen')
                f()
            })

        }

    }, []);

    return (
        <BrowserRouter>
        <Layout hasSider>
            <MySider  collapsed={collapsed} menuStore={menuStore}/>
            <Layout>
                <Header style={{ padding: 0, background: colorBgContainer }} >
                    <Flex vertical={false} style={{alignItems:"center"}}>
                        <Button
                            type="text"
                            icon={collapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
                            onClick={() => setCollapsed(!collapsed)}
                            style={{
                                borderColor: "transparent",
                                fontSize: '16px',
                                width: 64,
                                height: 64,
                            }}
                        />
                        <Stock_search searchStore={searchStore} />
                    </Flex>
                </Header>

                <Content style={{ margin: '24px 16px 0', overflow: 'initial' }}>
                    <div
                        style={{
                            padding: 24,
                            overflow: 'initial',
                            background: colorBgContainer,
                            borderRadius: borderRadiusLG,
                        }}
                    >
                        <Routes>
                            <Route path='/' element={<StockTable   />} />
                            <Route path='/stocklist' element={<StockTable/>} />
                            <Route path="/kline"  element={<KLineChart menuCollapsed={collapsed} /> } />
                            <Route path="/rt"  element={<RTKLineChart  menuCollapsed={collapsed}/>} />
                        </Routes>
                    </div>
                </Content>

                <Footer style={{ textAlign: 'center' }}>
                    Ant Design ©{new Date().getFullYear()} Created by Ant UED
                </Footer>
            </Layout>
        </Layout>
        </BrowserRouter>
    );
};

export default App;