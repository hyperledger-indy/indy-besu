Install k3s + Rancher with letsecrypt

```
sudo su
```


```
export IP_ADDRESS_HOST=3.215.23.96

dnf install htop -y
curl -sfL https://get.k3s.io | sh -
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
alias k=kubectl
export KUBECONFIG=/etc/rancher/k3s/k3s.yaml

helm repo add jetstack https://charts.jetstack.io
helm install cert-manager jetstack/cert-manager \
  --namespace cert-manager \
  --create-namespace \
  --set crds.enabled=true

helm repo add rancher-stable https://releases.rancher.com/server-charts/stable
kubectl create namespace cattle-system

helm upgrade -i rancher rancher-stable/rancher \
  --namespace cattle-system \
  --set hostname=$IP_ADDRESS_HOST.sslip.io \
  --set bootstrapPassword=admin \
  --set ingress.tls.source=letsEncrypt \
  --set letsEncrypt.email=mirafzal.shavkatov@dsr-corporation.com \
  --set letsEncrypt.ingress.class=traefik \
  --set replicas=1 \
  --set agentTLSMode=system-store

echo https://$IP_ADDRESS_HOST.sslip.io/dashboard/?setup=$(kubectl get secret --namespace cattle-system bootstrap-secret -o go-template='{{.data.bootstrapPassword|base64decode}}')

```

Install k3s + Rancher self-signed certificate

```
sudo su

dnf install htop -y

curl -sfL https://get.k3s.io | sh -

curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

alias k=kubectl

export KUBECONFIG=/etc/rancher/k3s/k3s.yaml

helm repo add jetstack https://charts.jetstack.io
helm install cert-manager jetstack/cert-manager \
  --namespace cert-manager \
  --create-namespace \
  --set crds.enabled=true

helm repo add rancher-stable https://releases.rancher.com/server-charts/stable
kubectl create namespace cattle-system

helm upgrade -i rancher rancher-stable/rancher \
  --namespace cattle-system \
  --set hostname=ec2-54-166-76-61.compute-1.amazonaws.com \
  --set bootstrapPassword=admin \
  --set replicas=1

echo https://ec2-54-166-76-61.compute-1.amazonaws.com/dashboard/?setup=$(kubectl get secret --namespace cattle-system bootstrap-secret -o go-template='{{.data.bootstrapPassword|base64decode}}')
```

Install ebs-csi driver

```
helm upgrade --install aws-ebs-csi-driver \
    --namespace kube-system \
    aws-ebs-csi-driver/aws-ebs-csi-driver --values aws-ebs-csi-driver-values.yaml
```

Then apply aws-secret.yaml

```
kubectl apply -f aws-secret.yaml
```

Install besu
```
git clone https://github.com/ConsenSys/quorum-kubernetes.git
cd quorum-kubernetes/helm
kubectl create namespace besu
helm install genesis ./charts/besu-genesis --namespace besu --create-namespace --values ./values/genesis-besu.yml
helm install bootnode-1 ./charts/besu-node --namespace besu --values ./values/bootnode.yml
helm install bootnode-2 ./charts/besu-node --namespace besu --values ./values/bootnode.yml
helm install validator-1 ./charts/besu-node --namespace besu --values ./values/validator.yml
helm install validator-2 ./charts/besu-node --namespace besu --values ./values/validator.yml
```