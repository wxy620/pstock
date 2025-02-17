import React from 'react';
import {Col, Flex, Row} from "antd";
import KLineChart from "./klinechart.tsx";
import Top5Card from "./top5card.tsx";
// 引入样式


interface RTKProps{
    menuCollapsed: boolean;
}
const RTKLineChart: React.FC<RTKProps> = ({menuCollapsed}) => {

    return (
        <Row gutter={[8,8]}>
            <Col span={20}>
                <KLineChart menuCollapsed={menuCollapsed} />
            </Col>
            <Col span={4}>
                <Top5Card />
            </Col>
        </Row>
    )
}

export default RTKLineChart