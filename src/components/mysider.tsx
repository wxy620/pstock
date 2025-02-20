import React, {useEffect, useRef, useState} from "react";
import {Menu, MenuProps} from "antd";
import Sider from "antd/es/layout/Sider";
import {AppstoreOutlined, BarChartOutlined, CloudOutlined, TeamOutlined} from "@ant-design/icons";
import {Link} from "react-router-dom";
import {makeAutoObservable} from "mobx";
import {observer} from "mobx-react-lite";


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

export class MenuStore {
    selectKey: string[] = ['1']
    constructor() {
        makeAutoObservable(this)
    }

    setKey(value: string[]) {
        this.selectKey = value
    }
}


export const menuStore = new MenuStore()

const MySider = observer<{collapsed: boolean, menuStore: MenuStore}>((
    {collapsed, menuStore})=>{
    const siderRef = useRef(null);
    return(
        <>
            <Sider ref={siderRef}
                /** 自带的collapsed开关 **/
                   trigger={null}
                   collapsible collapsed={collapsed} style={siderStyle}>
                <div className="demo-logo-vertical" />
                <Menu theme="dark"  mode="inline" defaultSelectedKeys={menuStore.selectKey}
                      items={items} selectedKeys={menuStore.selectKey}
                      onSelect={(i)=>{
                          menuStore.setKey([i.key])
                      }}
                />
            </Sider>
        </>
    )
})


export default MySider
