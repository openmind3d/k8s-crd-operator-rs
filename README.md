# Crd operator

Simple example how to create crds operator with kube.rs

Define crd type
```sh
kubectl apply -f crd.yaml
```
Create crds
```sh
kubectl apply -f device_think.yaml
kubectl apply -f device_iphone.yaml
```