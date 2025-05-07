<script setup>
import { ref, nextTick, defineProps, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import Button from 'primevue/button';
import Textarea from 'primevue/textarea';
import DataView from 'primevue/dataview';
import Tag from 'primevue/tag';
import ScrollPanel from 'primevue/scrollpanel';
import InputText from 'primevue/inputtext';

const props = defineProps({
    serverPort: {
        type: Number,
        default: 9999
    },
    maxMessages: {
        type: Number,
        default: 1000
    }
});

const msg = ref("");
const ips = ref("");
const port = ref(props.serverPort);
const chatting_ls = ref([]);
const scrollChatRef = ref(null);

// 滚动到底部方法
const scrollToBottomChatting = () => {
    nextTick(() => {
        const container = scrollChatRef.value?.$el.querySelector('.p-scrollpanel-content')
        if (container) {
            container.scrollTo({
                top: container.scrollHeight,
                behavior: 'smooth'
            })
        }
    })
}

async function open() {
    console.log("打开", "Port:", port.value);
    if (typeof invoke === 'function') { // 检查 invoke 是否存在
        try {
            let res = await invoke('start_ws_server', { port: port.value });
            console.log("Response from server:", res);
        } catch (error) {
            console.error('invoke 调用失败:', error);
        }
    } else {
        console.error('invoke 函数未正确加载');
    }
}

async function send() {
    console.log("发送");
    if (typeof invoke === 'function') { // 检查 invoke 是否存在
        try {
            let res = await invoke('broadcast_message', { message: msg.value });
            console.log(res);
        } catch (error) {
            console.error('invoke 调用失败:', error);
        }
    } else {
        console.error('invoke 函数未正确加载');
    }
}

const getItemClass = (tag) => {
    return tag === 'send' ? 'text-right' : 'text-left';
};

const formatData = (data) => {
    try {
        console.log(data);
        // 尝试解析为 JSON
        const parsedData = JSON.parse(data);
        // 如果解析成功，将其转换为格式化后的 JSON 字符串
        return JSON.stringify(parsedData, null, 2);
    } catch (error) {
        // 如果解析失败，说明不是有效的 JSON 字符串，直接返回原数据
        console.log(error);
        return data;
    }
};

const showMessageContent = (content) => {
    msg.value = formatData(content);
    send();
};

onMounted(() => {
    // 监听后端事件
    listen('ws_server', (event) => {
        chatting_ls.value.push({ tag: "info", data: event.payload });
        scrollToBottomChatting();
    }).then(() => {
    }).catch((error) => {
        console.error('监听 ws_server 事件失败:', error);
    });

    listen('ws_accept', (event) => {
        chatting_ls.value.push({ tag: "accept", data: event.payload });
        scrollToBottomChatting();
    }).then(() => {
    }).catch((error) => {
        console.error('监听 ws_accept 事件失败:', error);
    });

    listen('ws_send', (event) => {
        chatting_ls.value.push({ tag: "send", data: event.payload });
        scrollToBottomChatting();
    }).then(() => {
    }).catch((error) => {
        console.error('监听 ws_send 事件失败:', error);
    });

    listen('ws_ips', (event) => {
        ips.value = event.payload;
    }).then(() => {
    }).catch((error) => {
        console.error('监听 ws_ips 事件失败:', error);
    });

    // 禁止鼠标右键
    document.addEventListener('contextmenu', (e) => {
        e.preventDefault();
    });

    // 禁止 F5 刷新
    document.addEventListener('keydown', (e) => {
        if (e.key === 'F5' || (e.ctrlKey && e.key === 'r') || (e.metaKey && e.key === 'r')) {
            e.preventDefault();
        }
    });

});

onUnmounted(() => {
    // 在组件卸载时移除事件监听器，避免内存泄漏
    document.removeEventListener('contextmenu', (e) => {
        e.preventDefault();
    });

    document.removeEventListener('keydown', (e) => {
        if (e.key === 'F5' || (e.ctrlKey && e.key === 'r') || (e.metaKey && e.key === 'r')) {
            e.preventDefault();
        }
    });
});

const getSeverity = (tag) => {
    const severityMap = {
        'send': 'primary',
        'accept': 'info',
        'info': 'warn'
    }
    return severityMap[tag] || 'info' // 默认值
}

defineExpose({
    connect: open,
    sendMessage: send
});
</script>

<template>
    {{ ips }}
    <div class="flex items-center p-1">
        <InputText style="width: 50%;" type="text" v-model="port" class="mt-0 ml-0"></InputText>
        <Button @click="open" style="width: 50%;" class="ml-2 ">连接</Button>
    </div>

    <div style="width: 99%;">
        <Textarea v-model="msg" rows="1" fluid />
        <Button @click="send" fluid>发送</Button>
    </div>
    <div>
        <DataView :value="chatting_ls">
            <template #empty>
                <div></div>
            </template>
            <template #list="slotProps">
                <ScrollPanel style="width: 100%; height: 45vh" ref="scrollChatRef">
                    <div v-for="(item, index) in slotProps.items" :key="index" :class="getItemClass(item.tag)">
                        <Tag :severity="getSeverity(item.tag)" :value="item.data"
                            @dblclick="showMessageContent(item.data)" class="mt-1" />
                    </div>
                </ScrollPanel>
            </template>
        </DataView>
    </div>
</template>

<style scoped>
.grid-datatable {
    width: auto;
}

.text-right {
    text-align: right;
}

.text-left {
    text-align: left;
}
</style>