import os
import file_
import collections
import shutil
import sys
#使用方法：python3 rm_oldfile.py 退休文件夹路径,如果需要改变退休文件夹的大小，调整max_memory的数值。
#例如：退休文件夹路径为：archive/main,则使用方法为：python3 rm_oldfile.py archive/main
def main(args):  #将该退休的deb整理成指定大小的文件夹
    if os.path.exists(args[0]):
        file_.rely_(args[0])   #建立包的依赖关系
        path=args[0]
        dir_list=[]
        max_memory=25*1000*1000    #设置指定大小
        for file in os.listdir(path):
            dir_list.append(path+'/'+file)
            dir_dic=collections.OrderedDict()
            dir_list.sort()
        for i in dir_list:
            for dir in os.listdir(i):
                dir_dic[dir]=i+'/'+dir

        dir_memory={}
        for key in dir_dic:
            dir_memory[key]=file_.get_real_size(dir_dic[key])
        n=0
        while dir_dic:   //移动该退休的文件夹
            os.makedirs(path + '/old_deb' + str(n))

            for key in list(dir_dic.keys()):
                if file_.get_real_size(path+'/old_deb'+str(n))+dir_memory[key]<max_memory:
                    if os.path.exists(dir_dic[key]):
                        if os.path.exists(path+'/old_deb'+str(n)+'/Repository/stable/main/'+dir_dic[key].split(sep='/')[-2]):
                            for i in os.listdir(dir_dic[key]):
                                shutil.move(dir_dic[key]+'/'+i,path+'/old_deb'+str(n)+'/Repository/stable/main/'+dir_dic[key].split(sep='/')[-2]+'/'+i)
                            del dir_dic[key]
                        else:
                            if os.path.isfile(dir_dic[key]):
                                    os.makedirs(path + '/old_deb' + str(n) + '/Repository/stable/main/' +dir_dic[key].split(sep='/')[-2])
                                    shutil.move(dir_dic[key],path + '/old_deb' + str(n) + '/Repository/stable/main/' + dir_dic[key].split(sep='/')[-2] + '/' + key)
                                    del dir_dic[key]
                            else:
                                os.makedirs(path+'/old_deb'+str(n)+'/Repository/stable/main/'+dir_dic[key].split(sep='/')[-2])
                                for i in os.listdir(dir_dic[key]):
                                    shutil.move(dir_dic[key]+'/'+i,path + '/old_deb' + str(n) + '/Repository/stable/main/' + dir_dic[key].split(sep='/')[-2] + '/' + i)
                        #print(path + '/old_deb' + str(n) + '/' + dir_dic[key].split(sep='/')[-2] + '/' + key)
                                del dir_dic[key]
            n=n+1
        for i in os.listdir(path):
            file_.del_empty_file(path+'/'+i)
        file_.del_empty_file(path)

if __name__=='__main__':
    main(sys.argv[1:])
