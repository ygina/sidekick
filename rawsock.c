// https://levelup.gitconnected.com/write-a-linux-packet-sniffer-from-scratch-with-raw-socket-and-bpf-c53734b51850
#include <unistd.h>
#include <stdlib.h>
#include <stdio.h>
#include <arpa/inet.h>
#include <net/ethernet.h>
#include <sys/ioctl.h>
#include <net/if.h>
#include <string.h>

#define MAX_ITERS 10
#define BUFFER_SIZE 46

void main(int argc, char ** argv)
{
	// Create a socket
	int sock;
	if ((sock = socket(PF_PACKET, SOCK_RAW, htons(ETH_P_IP))) < 0) {
		perror("socket");
		exit(1);
	}
	printf("sock = %d\n", sock);

	// Bind the sniffer to a specific interface
	const char *opt = "h1-eth0";
	if (setsockopt(sock, SOL_SOCKET, SO_BINDTODEVICE, opt, strlen(opt) + 1) < 0) {
		perror("setsockopt");
		close(sock);
		exit(1);
	}

	// Set the network card in promiscuous mode
	struct ifreq ethreq;
	strncpy(ethreq.ifr_name, opt, IF_NAMESIZE);
	if (ioctl(sock, SIOCGIFFLAGS, &ethreq) == -1) {
		perror("ioctl 1");
		close(sock);
		exit(1);
	}
	ethreq.ifr_flags |= IFF_PROMISC;
	if (ioctl(sock, SIOCSIFFLAGS, &ethreq) == -1) {
		perror("ioctl 2");
		close(sock);
		exit(1);
	}

	int n;
	char buffer[BUFFER_SIZE];
	for (int i = 0; i < MAX_ITERS; i++) {
		n = recv(sock, buffer, BUFFER_SIZE, 0);

		// Packet contains at least Ethernet (14), IP (20),
		// and TCP/UDP (8) headers
		if (n < 42) {
			perror("recvfrom");
			close(sock);
			exit(0);
		}
		printf(
			"%d bytes: [%d %d %d %d]\n",
			n - 42,
			buffer[42],
			buffer[43],
			buffer[44],
			buffer[45]
		);
		// printf(".");
	}
	printf("done.\n");
}

