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
import {Link, BrowserRouter, Routes, Route} from "react-router-dom";

import KLineChart from "./components/klinechart.tsx";
import RTKLineChart from "./components/rt-klinechart.tsx";
import StockTable from "./components/stocktable.tsx";

import {emit, listen} from "@tauri-apps/api/event";
import {IPayload} from "./model/payload.ts";
import {invoke} from "@tauri-apps/api/core";
import Stock_search , {searchStore} from "./components/stock_search.tsx";


const { Header, Content, Footer, Sider } = Layout;

const siderStyle: React.CSSProperties = {
    overflow: 'auto',
    height: 'auto',
    position: 'initial',
    insetInlineStart: 0,
    top: 0,
    bottom: 0,
    scrollbarWidth: 'thin',
    scrollbarGutter: 'stable',
};


type MenuItem = Required<MenuProps>['items'][number];
const items: MenuItem[] = [
    { key: '1', icon: <BarChartOutlined />, label: <Link to='/stocklist'>列表</Link>,},
    { key: '2', icon: <CloudOutlined />, label: <Link to='/kline'>个股</Link>,  },
    { key: '3', icon: <AppstoreOutlined />, label: <Link to='/rt'>分时</Link> },
    {
        key: 'sub1',
        label: 'Navigation One',
        icon: <TeamOutlined />,
        children: [
            { key: '5', label: 'Option 5' },
            { key: '6', label: 'Option 6' },
            { key: '7', label: 'Option 7' },
            { key: '8', label: 'Option 8' },
        ],
    },
    {
        key: 'sub2',
        label: 'Navigation Two',
        icon: <AppstoreOutlined />,
        children: [
            { key: '9', label: 'Option 9' },
            { key: '10', label: 'Option 10' },
            {
                key: 'sub3',
                label: 'Submenu',
                children: [
                    { key: '11', label: 'Option 11' },
                    { key: '12', label: 'Option 12' },
                ],
            },
        ],
    },
];

const App: React.FC = () => {
    const [current, setCurrent] = useState('1');
    const [collapsed, setCollapsed] = useState(true);
    const {
        token: { colorBgContainer, borderRadiusLG },
    } = theme.useToken();

    const siderRef = useRef(null);


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
            <Sider ref={siderRef}
                /** 自带的collapsed开关 **/
                   trigger={null}
                   collapsible collapsed={collapsed} style={siderStyle}  onCollapse={(value)=> setCollapsed(value)}>
                <div className="demo-logo-vertical" />
                    <Menu theme="dark"  mode="inline" defaultSelectedKeys={[current]}  items={items} />
            </Sider>
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