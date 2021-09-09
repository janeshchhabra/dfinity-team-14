# Boot From a Canister

The idea is that a developer (or an enterprise) can boot an OS by uploading it to an IC Canister. 

- A virtual disk (An Ubuntu-minimal PXE image in our example) is uploaded to an IC Canister.
- A target host talks to the DHCP server to get the location of the PXE image (ie TFTP server)
- TFTP Server interacts with ic-agent that extracts the image from the canister
- On the target host, using PXE, a guest VM will be allowed to boot and load its configuration from the TXE server.

Note: An Ubuntu iso is uploaded to an asset canister at https://lk5lj-2iaaa-aaaah-aaraa-cai.raw.ic0.app/ubuntu.iso 

Note: We got the tftp server from https://github.com/DarinM223/tftp-server 

Steps for the demo:
1. Upload an Ubuntu iso to the canister using the uplaoder (Details to come)
2. On the host, create tftp folder
```
$ sudo mkdir -p /srv/tftp
$ sudo chmod -R 777 /srv/tftp
$ sudo chown -R nobody: /srv/tftp
```
2. Get the Rust tftp server and start it on localhost:69 and point it to /srv/tftp
````
$ git clone https://github.com/DarinM223/tftp-server 
$ cd tftp-server
$ cargo build --example server 
$ ./target/debug/examples/server -p 69 -d /srv/tftp
Server created at address V4(127.0.0.1:69)
````
3. Check that tftp is listening on local port 69
````
$ nc -uvz 127.0.0.1 69
Connection to 127.0.0.1 69 port [udp/tftp] succeeded!
````
4. Prepare libvirt network to provide PXE
````
$ virsh net-dumpxml default > net-default.xml
$ virsh net-edit default
<network>
 <name>default</name>
 <uuid>cfa3d2de-4c67-4f8f-91eb-69502e59e64d</uuid>
 <forward mode='nat'/>
 <bridge name='virbr0' stp='on' delay='0'/>
 <mac address='52:54:00:08:6f:33'/>
 <ip address='192.168.122.1' netmask='255.255.255.0'>
 <strong><tftp root='/srv/tftp'/></strong>
  <dhcp>
   <range start='192.168.122.2' end='192.168.122.254'/>
   <strong><bootp file='pxelinux.0' server='0.0.0.0'/></strong>
  </dhcp>
 </ip>
</network>
</pre>
````
5. And restart the virtual network again:
````
$ virsh net-destroy default
Network default destroyed

$ virsh net-start default
Network default started
````
6. Make sure that /srv/tftp has the following (mount iso and get these files)
````
$ sudo cp -v /usr/lib/PXELINUX/pxelinux.0 /srv/tftp/
````
copy ldlinux.c32, libcom32.c32, libutil.c32, vesamenu.c32, intird.img and vmlinuz files to the /srv/tftp directory. Setup /srv/tftp/pxelinux.cfg/default

7. Now install the image using virt-manager
TFTP server will pull the image from IC.
