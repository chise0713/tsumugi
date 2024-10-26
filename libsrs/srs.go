package main

/*
#include <stdint.h>
#include <stdlib.h>

typedef struct {
    uint8_t ip[4];
    uint8_t prefix;
} IPv4CIDR;

typedef struct {
    uint8_t ip[16];
    uint8_t prefix;
} IPv6CIDR;

typedef struct {
    IPv4CIDR* ipv4_list;
	uint32_t ipv4_count;
    IPv6CIDR* ipv6_list;
	uint32_t ipv6_count;
} CIDRList;
*/
import "C"
import (
	"bytes"
	"fmt"
	"net"
	"unsafe"

	"github.com/sagernet/sing-box/common/srs"
	"github.com/sagernet/sing-box/constant"
	"github.com/sagernet/sing-box/option"
)

func read(data *C.uint8_t, length C.uint32_t) (option.PlainRuleSet, error) {
	slice := C.GoBytes(unsafe.Pointer(data), C.int(length))
	reader := bytes.NewReader(slice)
	ruleset, err := srs.Read(reader, true)
	if err != nil {
		return option.PlainRuleSet{}, err
	}
	return ruleset, nil
}

func write(rule_set option.PlainRuleSet, length *C.uint32_t) *C.uint8_t {
	buffer := bytes.NewBuffer(nil)
	if err := srs.Write(buffer, rule_set); err != nil {
		return (*C.uint8_t)(C.NULL)
	}
	data := buffer.Bytes()
	if len(data) == 0 {
		return (*C.uint8_t)(C.NULL)
	}
	cData := C.CBytes(data)
	*length = C.uint32_t(len(data))
	return (*C.uint8_t)(cData)
}

//export read_cidr_rule
func read_cidr_rule(data *C.uint8_t, length C.uint32_t) *C.CIDRList {
	ruleset, err := read(data, length)
	if err != nil {
		return (*C.CIDRList)(C.NULL)
	}
	cidr_list := (*C.CIDRList)(C.malloc(C.sizeof_CIDRList))
	var ipv4_list []C.IPv4CIDR
	var ipv6_list []C.IPv6CIDR
	for _, rule := range ruleset.Rules {
		for _, cidr_str := range rule.DefaultOptions.IPCIDR {
			_, ipnet, _ := net.ParseCIDR(cidr_str)
			ip := ipnet.IP
			prefix_len, _ := ipnet.Mask.Size()

			if ip.To4() != nil {
				ipv4 := C.IPv4CIDR{}
				for i, b := range ip.To4() {
					ipv4.ip[i] = C.uint8_t(b)
				}
				ipv4.prefix = C.uint8_t(prefix_len)
				ipv4_list = append(ipv4_list, ipv4)
			} else {
				ipv6 := C.IPv6CIDR{}
				for i, b := range ip.To16() {
					ipv6.ip[i] = C.uint8_t(b)
				}
				ipv6.prefix = C.uint8_t(prefix_len)
				ipv6_list = append(ipv6_list, ipv6)
			}
		}
	}
	cidr_list.ipv4_count = C.uint32_t(len(ipv4_list))
	cidr_list.ipv6_count = C.uint32_t(len(ipv6_list))
	cidr_list.ipv4_list = (*C.IPv4CIDR)(C.malloc(C.size_t(len(ipv4_list)) * C.sizeof_IPv4CIDR))
	cidr_list.ipv6_list = (*C.IPv6CIDR)(C.malloc(C.size_t(len(ipv6_list)) * C.sizeof_IPv6CIDR))

	for i, ipv4 := range ipv4_list {
		*(*C.IPv4CIDR)(unsafe.Pointer(uintptr(unsafe.Pointer(cidr_list.ipv4_list)) + uintptr(i)*C.sizeof_IPv4CIDR)) = ipv4
	}
	for i, ipv6 := range ipv6_list {
		*(*C.IPv6CIDR)(unsafe.Pointer(uintptr(unsafe.Pointer(cidr_list.ipv6_list)) + uintptr(i)*C.sizeof_IPv6CIDR)) = ipv6
	}

	return cidr_list
}

//export write_cidr_rule
func write_cidr_rule(cidr_list *C.CIDRList, length *C.uint32_t) *C.uint8_t {
	var rule option.DefaultHeadlessRule
	for i := 0; i < int(cidr_list.ipv4_count); i++ {
		ipv4 := (*C.IPv4CIDR)(unsafe.Pointer(uintptr(unsafe.Pointer(cidr_list.ipv4_list)) + uintptr(i)*unsafe.Sizeof(*cidr_list.ipv4_list)))
		ip := net.IP(C.GoBytes(unsafe.Pointer(&ipv4.ip), C.int(len(ipv4.ip))))
		prefix := int(ipv4.prefix)
		cidr_str := fmt.Sprintf("%s/%d", ip.String(), prefix)
		rule.IPCIDR = append(rule.IPCIDR, cidr_str)
	}

	for i := 0; i < int(cidr_list.ipv6_count); i++ {
		ipv6 := (*C.IPv6CIDR)(unsafe.Pointer(uintptr(unsafe.Pointer(cidr_list.ipv6_list)) + uintptr(i)*unsafe.Sizeof(*cidr_list.ipv6_list)))
		ip := net.IP(C.GoBytes(unsafe.Pointer(&ipv6.ip), C.int(len(ipv6.ip))))
		prefix := int(ipv6.prefix)
		cidr_str := fmt.Sprintf("%s/%d", ip.String(), prefix)
		rule.IPCIDR = append(rule.IPCIDR, cidr_str)
	}

	var rule_set option.PlainRuleSet
	rule_set.Rules = []option.HeadlessRule{
		{
			Type:           constant.RuleTypeDefault,
			DefaultOptions: rule,
		},
	}
	return write(rule_set, length)
}

//export read_domain_rule
func read_domain_rule(data *C.uint8_t, length C.uint32_t) **C.char {
	ruleset, err := read(data, length)
	if err != nil {
		return (**C.char)(C.NULL)
	}

	var domains []*C.char

	for _, rule := range ruleset.Rules {
		for _, domain := range rule.DefaultOptions.Domain {
			cDomain := C.CString(domain)
			domains = append(domains, cDomain)
		}
	}

	domains_ptr := C.malloc(C.size_t(len(domains)) * C.size_t(unsafe.Sizeof(uintptr(0))))

	for i, domain := range domains {
		*(**C.char)(unsafe.Pointer(uintptr(domains_ptr) + uintptr(i)*unsafe.Sizeof(uintptr(0)))) = domain
	}

	return (**C.char)(domains_ptr)
}

//export write_domain_rule
func write_domain_rule(domains_ptr **C.char, length *C.uint32_t) *C.uint8_t {
	var rule option.DefaultHeadlessRule

	for i := 0; i < int(*length); i++ {
		domain := *(**C.char)(unsafe.Pointer(uintptr(unsafe.Pointer(domains_ptr)) + uintptr(i)*unsafe.Sizeof(uintptr(0))))
		rule.Domain = append(rule.Domain, C.GoString(domain))
	}

	var rule_set option.PlainRuleSet
	rule_set.Rules = []option.HeadlessRule{
		{
			Type:           constant.RuleTypeDefault,
			DefaultOptions: rule,
		},
	}
	return write(rule_set, length)
}

func main() {}
