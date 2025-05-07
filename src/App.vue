<template>
  <div class="card">
    <Tabs value="0">
      <TabList>
        <Tab value="0">Client</Tab>
        <Tab value="1">Server</Tab>
      </TabList>
      <TabPanels>
        <TabPanel value="0">
          <WebSocketClient server-ip="172.168.54.182" :server-port="9999" />
        </TabPanel>
        <TabPanel value="1">
          <WebSocketServer :server-port="9999" />
        </TabPanel>
      </TabPanels>
    </Tabs>
  </div>
</template>


<script setup>
import WebSocketClient from './components/WebSocketClient.vue';
import WebSocketServer from './components/WebSocketServer.vue';
import Tabs from 'primevue/tabs';
import TabList from 'primevue/tablist';
import Tab from 'primevue/tab';
import TabPanels from 'primevue/tabpanels';
import TabPanel from 'primevue/tabpanel';

</script>