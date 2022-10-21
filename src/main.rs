use anyhow::{anyhow, Result};
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{
        Api, ApiResource, DeleteParams, DynamicObject, GroupVersionKind, ListParams, Patch,
        PatchParams, PostParams, WatchEvent,
    },
    core::ObjectList,
    runtime::{
        wait::{await_condition, conditions},
        watcher, WatchStreamExt,
    },
    Client, CustomResource, CustomResourceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{rc::Rc, sync::Arc};

#[derive(
    CustomResource, Serialize, Deserialize, Default, Debug, PartialEq, Eq, Clone, JsonSchema,
)]
#[kube(
    group = "device.crd",
    version = "v1",
    kind = "Device",
    plural = "device",
    namespaced
)]
struct DeviceSpec {
    device_type: String,
    name: String,
    vendor: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let custom_operator = KubeOperator::new().await?;

    // for ele in custom_operator.list_devices().await.unwrap().items {
    //     println!("{:?}", ele);
    // }

    // let think = Device::new(
    //     "think",
    //     DeviceSpec {
    //         device_type: "laptop".to_string(),
    //         name: "thinkpad t480s".to_string(),
    //         vendor: "lenovo".to_string(),
    //     },
    // );

    // custom_operator.create_device(&think).await?;

    // custom_operator.watch_devices().await;

    let custom_operator = Arc::new(custom_operator);
    tokio::spawn(async move { custom_operator.watch_devices().await });

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    println!("done");

    Ok(())
}

struct KubeOperator {
    client: Client,
    device_api: Api<Device>,
}

impl KubeOperator {
    pub async fn new() -> Result<Self> {
        let client = Client::try_default().await?;
        let device_api = Api::<Device>::default_namespaced(client.clone());
        Ok(Self { client, device_api })
    }

    pub async fn list_devices(&self) -> Result<ObjectList<Device>, kube::Error> {
        self.device_api.list(&ListParams::default()).await
    }

    pub async fn create_device(&self, device: &Device) -> Result<()> {
        match self.device_api.create(&PostParams::default(), device).await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!(format!("failed to create device: {}", e))),
        }
    }

    pub async fn delete_device(&self, name: &str) -> Result<()> {
        match self.device_api.delete(name, &DeleteParams::default()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!(format!("failed to delete device: {}", e))),
        }
    }

    pub async fn watch_devices(&self) {
        watcher(self.device_api.clone(), ListParams::default())
            .applied_objects()
            .try_for_each(|device| async move {
                println!(
                    "new device! name: {:?}, device_type: {}, vendor: {}",
                    device.metadata.name.ok_or(""),
                    device.spec.device_type,
                    device.spec.vendor
                );
                Ok(())
            })
            .await
            .unwrap();
    }
}
