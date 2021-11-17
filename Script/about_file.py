import os
import file_
import sys
#使用方法：python3 about_file.py 退休文件夹路径
#例如：退休文件夹路径为：archive/main/old_deb0,则使用方法为：python3 about_file.py archive/main/old_deb0
def main(args):   #计算文件的md5和tree 
    if os.path.exists(args[0]):
        savedStdout=sys.stdout
        path=args[0]
        file_dict = {}
        #new_list=os.listdir(path)
        os.chdir(path)
        file_.get_filedict('Repository',file_dict)
        filedict=sorted(file_dict.items(),reverse=False)
        print_log = open('md5', 'a')
        sys.stdout = print_log
        for key in filedict:
            print(file_.md5sum(key[1])+' '+key[1])
        print_log.close()
        print_log=open('tree','a')
        sys.stdout=print_log
        file_tree=os.popen('tree '+'Repository').readlines()
        for i in range(len(file_tree)-1):
            print(file_tree[i][:-1])
        print_log.close()
        sys.stdout=savedStdout

if __name__=='__main__':
    main(sys.argv[1:])
