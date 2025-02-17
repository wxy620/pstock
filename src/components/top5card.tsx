import React from 'react';
import type { CollapseProps } from 'antd';
import { Collapse } from 'antd';


const text = `
  A dog is a type of domesticated animal.
  Known for its loyalty and faithfulness,
  it can be found as a welcome guest in many households across the world.
`;

const items: CollapseProps['items'] = [
    {
        key: '1',
        label: '买卖5档',
        children: <p>{text}</p>,
    },
    {
        key: '2',
        label: '分时成交',
        children: <p>{text}</p>,
    },
];

const Top5Card: React.FC = () => {

    return (
        <Collapse accordion items={items}  defaultActiveKey={[1]} />
    )
}

export default Top5Card